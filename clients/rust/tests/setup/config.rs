#![cfg(feature = "test-sbf")]
#![allow(dead_code)]

use paladin_rewards_program_client::accounts::HolderRewards;
use paladin_stake_program_client::{
    accounts::Config, instructions::InitializeConfigBuilder, pdas::find_vault_pda,
};
use solana_program_test::{BanksClientError, ProgramTestContext};
use solana_sdk::{
    pubkey::Pubkey, signature::Keypair, signer::Signer, system_instruction,
    system_transaction::transfer, transaction::Transaction,
};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};

use crate::setup::rewards::RewardsManager;

use super::token::create_mint;

pub struct ConfigManager {
    // Config account.
    pub config: Pubkey,
    // Config authority.
    pub config_authority: Keypair,
    // Vault token account.
    pub vault: Pubkey,
    // Vault token account.
    pub vault_pda: Pubkey,
    // Vault token account.
    pub vault_holder_rewards: Pubkey,
    // Mint account.
    pub mint: Pubkey,
    // Mint authority.
    pub mint_authority: Keypair,
    pub rewards_manager: RewardsManager,
}

impl ConfigManager {
    pub async fn new(context: &mut ProgramTestContext) -> Self {
        // cooldown_time_seconds = 1 second
        // max_deactivation_basis_points = 500 (5%)
        // sync_rewards_lamports = 1_000_000 (0.001 SOL)
        Self::with_args(context, 1, 500, 1_000_000).await
    }

    pub async fn with_args(
        mut context: &mut ProgramTestContext,
        cooldown_time_seconds: u64,
        max_deactivation_basis_points: u16,
        sync_rewards_lamports: u64,
    ) -> Self {
        // Creates the mint.
        let mint = Keypair::new();
        let mint_authority = Keypair::new();
        let config = Keypair::new();

        create_mint(
            context,
            &mint,
            &mint_authority.pubkey(),
            Some(&mint_authority.pubkey()),
            0,
        )
        .await
        .unwrap();
        let (vault_pda, _) = find_vault_pda(&config.pubkey());

        let rewards_manager = RewardsManager::new(&mut context, &mint.pubkey(), &vault_pda).await;

        let mut manager = ConfigManager {
            config: config.pubkey(),
            config_authority: Keypair::new(),
            vault: Pubkey::default(),
            vault_pda: Pubkey::default(),
            vault_holder_rewards: Pubkey::default(),
            mint: mint.pubkey(),
            mint_authority,
            rewards_manager,
        };

        // Create vault
        let vault = get_associated_token_address(&vault_pda, &mint.pubkey());
        manager.vault_pda = vault_pda;
        manager.vault = vault;

        // Create the holder rewards account for the vault.
        let (vault_holder_rewards, _) = HolderRewards::find_pda(&vault_pda);
        manager.vault_holder_rewards = vault_holder_rewards;

        // Fund vault pda
        fund_account(&mut context, &vault_pda, 0).await.unwrap();
        // Fund vault holder rewards
        fund_account(&mut context, &vault_holder_rewards, HolderRewards::LEN)
            .await
            .unwrap();
        // create vault ATA
        create_ata(&mut context, &vault_pda, &mint.pubkey())
            .await
            .unwrap();

        // Initializes the config.
        let create_ix = system_instruction::create_account(
            &context.payer.pubkey(),
            &config.pubkey(),
            context
                .banks_client
                .get_rent()
                .await
                .unwrap()
                .minimum_balance(Config::LEN),
            Config::LEN as u64,
            &paladin_stake_program_client::ID,
        );
        let initialize_ix = InitializeConfigBuilder::new()
            .config(config.pubkey())
            .mint(mint.pubkey())
            .holder_rewards_pool(manager.rewards_manager.pool)
            .holder_rewards_pool_token_account(manager.rewards_manager.pool_token_account)
            .vault(manager.vault)
            .vault_pda(vault_pda)
            .vault_holder_rewards(vault_holder_rewards)
            .rewards_program(paladin_rewards_program_client::ID)
            .config_authority(manager.config_authority.pubkey())
            .slash_authority(manager.config_authority.pubkey())
            .cooldown_time_seconds(cooldown_time_seconds)
            .max_deactivation_basis_points(max_deactivation_basis_points)
            .sync_rewards_lamports(sync_rewards_lamports)
            .duna_document_hash(get_duna_hash())
            .instruction();

        context.get_new_latest_blockhash().await.unwrap();

        let tx = Transaction::new_signed_with_payer(
            &[create_ix, initialize_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &config],
            context.last_blockhash,
        );
        context.banks_client.process_transaction(tx).await.unwrap();

        manager
    }
}

pub async fn create_config(context: &mut ProgramTestContext) -> Pubkey {
    // cooldown_time_seconds = 1 second
    // max_deactivation_basis_points = 500 (5%)
    // sync_rewards_lamports = 1_000_000 (0.001 SOL)
    create_config_with_args(context, 1, 500, 1_000_000).await
}

pub async fn create_config_with_args(
    context: &mut ProgramTestContext,
    cooldown_time_seconds: u64,
    max_deactivation_basis_points: u16,
    sync_rewards_lamports: u64,
) -> Pubkey {
    let manager = ConfigManager::with_args(
        context,
        cooldown_time_seconds,
        max_deactivation_basis_points,
        sync_rewards_lamports,
    )
    .await;
    manager.config
}

pub fn get_duna_hash() -> [u8; 32] {
    let base64_doc_hash =
        b"IlRoaXMgaXMgdGhlIERBTyBjb25zdGl0dXRpb24sIGJ5IHNpZ25pbmcgdGhpcyBJIGFncmVlIHdpdGggaXQuIg==";

    let mut normalized_hash = [0u8; 32];

    // Simple XOR hash function
    for (i, &byte) in base64_doc_hash.iter().enumerate() {
        normalized_hash[i % 32] ^= byte;
    }

    normalized_hash
}

pub(crate) async fn fund_account(
    context: &mut ProgramTestContext,
    account: &Pubkey,
    data_len: usize,
) -> Result<(), BanksClientError> {
    context.get_new_latest_blockhash().await.unwrap();

    let tx = transfer(
        &context.payer,
        &account,
        context
            .banks_client
            .get_rent()
            .await
            .unwrap()
            .minimum_balance(data_len),
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

pub(crate) async fn create_ata(
    context: &mut ProgramTestContext,
    account: &Pubkey,
    mint: &Pubkey,
) -> Result<(), BanksClientError> {
    let ix =
        create_associated_token_account(&context.payer.pubkey(), account, mint, &spl_token::ID);

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}
