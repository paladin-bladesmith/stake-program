#![cfg(feature = "test-sbf")]
#![allow(dead_code)]

use paladin_stake::{
    accounts::Config, instructions::InitializeConfigBuilder, pdas::find_vault_pda,
};
use solana_program_test::ProgramTestContext;
use solana_sdk::{
    pubkey::Pubkey, signature::Keypair, signer::Signer, system_instruction,
    transaction::Transaction,
};

use super::token::{create_mint, create_token_account, MINT_EXTENSIONS, TOKEN_ACCOUNT_EXTENSIONS};

pub async fn create_config(context: &mut ProgramTestContext) -> Pubkey {
    // cooldown_time_seconds = 1 second
    // max_deactivation_basis_points = 500 (5%)
    create_config_with_args(context, 1, 500).await
}

pub async fn create_config_with_args(
    context: &mut ProgramTestContext,
    cooldown_time_seconds: u64,
    max_deactivation_basis_points: u16,
) -> Pubkey {
    let config = Keypair::new();
    let authority = Keypair::new().pubkey();

    let mint = Keypair::new();
    create_mint(
        context,
        &mint,
        &authority,
        Some(&authority),
        0,
        MINT_EXTENSIONS,
    )
    .await
    .unwrap();

    let token = Keypair::new();
    create_token_account(
        context,
        &find_vault_pda(&config.pubkey()).0,
        &token,
        &mint.pubkey(),
        TOKEN_ACCOUNT_EXTENSIONS,
    )
    .await
    .unwrap();

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
        &paladin_stake::ID,
    );

    let initialize_ix = InitializeConfigBuilder::new()
        .config(config.pubkey())
        .config_authority(authority)
        .slash_authority(authority)
        .mint(mint.pubkey())
        .vault(token.pubkey())
        .cooldown_time_seconds(cooldown_time_seconds)
        .max_deactivation_basis_points(max_deactivation_basis_points)
        .instruction();

    // When we create a config.

    let tx = Transaction::new_signed_with_payer(
        &[create_ix, initialize_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &config],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    config.pubkey()
}
