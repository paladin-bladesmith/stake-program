#![cfg(feature = "test-sbf")]

mod setup;

use paladin_stake::{
    accounts::Config, instructions::InitializeConfigBuilder, pdas::find_vault_pda,
    types::AccountType,
};
use setup::{create_mint, create_token};
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{
    signature::{Keypair, Signer},
    system_program,
    transaction::Transaction,
};
use spl_token_2022::extension::ExtensionType;

#[tokio::test]
async fn create() {
    let mut context = ProgramTest::new("stake_program", paladin_stake::ID, None)
        .start_with_context()
        .await;

    // Given an empty config account and a mint.

    let config = Keypair::new();
    let authority = Keypair::new().pubkey();

    let mint = Keypair::new();
    create_mint(&mut context, &mint, &authority, Some(&authority), 0, &[])
        .await
        .unwrap();

    let token = Keypair::new();
    create_token(
        &mut context,
        &find_vault_pda(&config.pubkey()).0,
        &token,
        &mint.pubkey(),
        &[ExtensionType::ImmutableOwner],
    )
    .await
    .unwrap();

    let ix = InitializeConfigBuilder::new()
        .config(config.pubkey())
        .config_authority(authority)
        .slash_authority(authority)
        .mint(mint.pubkey())
        .vault_token(token.pubkey())
        .payer(Some(context.payer.pubkey()))
        .system_program(Some(system_program::ID))
        .cooldown_time(1) // 1 second
        .max_deactivation_basis_points(500) // 5%
        .instruction();

    // When we create a config.

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &config],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then an account was created with the correct data.

    let account = context
        .banks_client
        .get_account(config.pubkey())
        .await
        .unwrap();

    assert!(account.is_some());

    let account = account.unwrap();
    assert_eq!(account.data.len(), Config::LEN);

    let account_data = account.data.as_ref();
    let counter = Config::from_bytes(account_data).unwrap();
    assert_eq!(counter.account_t_ype, AccountType::Config);
    assert_eq!(counter.slash_authority, authority);
    assert_eq!(counter.authority, authority);
}
