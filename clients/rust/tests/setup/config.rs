#![cfg(feature = "test-sbf")]
#![allow(dead_code)]

use paladin_stake_program_client::{
    accounts::Config, instructions::InitializeConfigBuilder, pdas::find_vault_pda,
};
use solana_program_test::ProgramTestContext;
use solana_sdk::{
    pubkey::Pubkey, signature::Keypair, signer::Signer, system_instruction,
    transaction::Transaction,
};

use super::token::{create_mint, create_token_account, MINT_EXTENSIONS, TOKEN_ACCOUNT_EXTENSIONS};

pub struct ConfigManager {
    // Config account.
    pub config: Pubkey,
    // Config authority.
    pub authority: Keypair,
    // Mint account.
    pub mint: Pubkey,
    // Mint authority.
    pub mint_authority: Keypair,
    // Vault token account.
    pub vault: Pubkey,
}

impl ConfigManager {
    pub async fn new(context: &mut ProgramTestContext) -> Self {
        // cooldown_time_seconds = 1 second
        // max_deactivation_basis_points = 500 (5%)
        // sync_rewards_lamports = 1_000_000 (0.001 SOL)
        Self::with_args(context, 1, 500, 1_000_000).await
    }

    pub async fn with_args(
        context: &mut ProgramTestContext,
        cooldown_time_seconds: u64,
        max_deactivation_basis_points: u16,
        sync_rewards_lamports: u64,
    ) -> Self {
        let mut manager = Self {
            config: Pubkey::default(),
            authority: Keypair::new(),
            mint: Pubkey::default(),
            mint_authority: Keypair::new(),
            vault: Pubkey::default(),
        };

        // creates the mint

        let mint = Keypair::new();
        create_mint(
            context,
            &mint,
            &manager.mint_authority.pubkey(),
            Some(&manager.mint_authority.pubkey()),
            0,
            MINT_EXTENSIONS,
        )
        .await
        .unwrap();

        manager.mint = mint.pubkey();

        // creates the valut (token account)

        let config = Keypair::new();
        let vault = Keypair::new();

        create_token_account(
            context,
            &find_vault_pda(&config.pubkey()).0,
            &vault,
            &manager.mint,
            TOKEN_ACCOUNT_EXTENSIONS,
        )
        .await
        .unwrap();

        manager.vault = vault.pubkey();

        // initializes the config

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
            .config_authority(manager.authority.pubkey())
            .slash_authority(manager.authority.pubkey())
            .mint(mint.pubkey())
            .vault(manager.vault)
            .cooldown_time_seconds(cooldown_time_seconds)
            .max_deactivation_basis_points(max_deactivation_basis_points)
            .sync_rewards_lamports(sync_rewards_lamports)
            .instruction();

        context.get_new_latest_blockhash().await.unwrap();

        let tx = Transaction::new_signed_with_payer(
            &[create_ix, initialize_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &config],
            context.last_blockhash,
        );
        context
            .banks_client
            .process_transaction_with_metadata(tx)
            .await
            .unwrap();

        manager.config = config.pubkey();
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
