#![cfg(feature = "test-sbf")]

mod setup;

use paladin_stake::{
    accounts::Config, errors::StakeError, instructions::InitializeConfigBuilder,
    pdas::find_vault_pda,
};
use setup::{create_mint, create_token, mint_to, MINT_EXTENSIONS, TOKEN_ACCOUNT_EXTENSIONS};
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};

mod initialize_config {

    use super::*;

    #[tokio::test]
    async fn initialize_config_with_mint_and_token() {
        let mut context = ProgramTest::new("stake_program", paladin_stake::ID, None)
            .start_with_context()
            .await;

        // Given an empty config account with an associated vault and a mint.

        let config = Keypair::new();
        let authority = Keypair::new().pubkey();

        let mint = Keypair::new();
        create_mint(
            &mut context,
            &mint,
            &authority,
            Some(&authority),
            0,
            MINT_EXTENSIONS,
        )
        .await
        .unwrap();

        let token = Keypair::new();
        create_token(
            &mut context,
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
            .cooldown_time_seconds(1) // 1 second
            .max_deactivation_basis_points(500) // 5%
            .instruction();

        // When we create a config.

        let tx = Transaction::new_signed_with_payer(
            &[create_ix, initialize_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &config],
            context.last_blockhash,
        );
        context.banks_client.process_transaction(tx).await.unwrap();

        // Then an account was created with the correct data.

        let account = get_account!(context, config.pubkey());
        assert_eq!(account.data.len(), Config::LEN);

        let account_data = account.data.as_ref();
        let config_account = Config::from_bytes(account_data).unwrap();
        assert_eq!(config_account.slash_authority, authority.into());
        assert_eq!(config_account.authority, authority.into());
    }

    #[tokio::test]
    async fn fail_initialize_config_with_wrong_token_authority() {
        let mut context = ProgramTest::new("stake_program", paladin_stake::ID, None)
            .start_with_context()
            .await;

        // Given an empty config account and a mint.

        let config = Keypair::new();
        let authority = Keypair::new().pubkey();

        let mint = Keypair::new();
        create_mint(
            &mut context,
            &mint,
            &authority,
            Some(&authority),
            0,
            MINT_EXTENSIONS,
        )
        .await
        .unwrap();

        let token = Keypair::new();
        create_token(
            &mut context,
            &authority, // <-- wrong authority
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
            .cooldown_time_seconds(1) // 1 second
            .max_deactivation_basis_points(500) // 5%
            .instruction();

        // When we try to initialize the config with the wrong token authority.

        let tx = Transaction::new_signed_with_payer(
            &[create_ix, initialize_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &config],
            context.last_blockhash,
        );

        let err = context
            .banks_client
            .process_transaction(tx)
            .await
            .unwrap_err();

        // Then we expect an error.

        assert_custom_error!(err, StakeError::InvalidTokenOwner);
    }

    #[tokio::test]
    async fn fail_initialize_config_with_non_empty_token() {
        let mut context = ProgramTest::new("stake_program", paladin_stake::ID, None)
            .start_with_context()
            .await;

        // Given an empty config account and a mint.

        let config = Keypair::new();
        let authority = Keypair::new();
        let authority_pubkey = authority.pubkey();

        let mint = Keypair::new();
        create_mint(
            &mut context,
            &mint,
            &authority_pubkey,
            Some(&authority_pubkey),
            0,
            MINT_EXTENSIONS,
        )
        .await
        .unwrap();

        let token = Keypair::new();
        create_token(
            &mut context,
            &find_vault_pda(&config.pubkey()).0,
            &token,
            &mint.pubkey(),
            TOKEN_ACCOUNT_EXTENSIONS,
        )
        .await
        .unwrap();

        // And we mint a token.

        mint_to(&mut context, &mint, &authority, &token.pubkey(), 1, 0)
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

        let intialize_ix = InitializeConfigBuilder::new()
            .config(config.pubkey())
            .config_authority(authority_pubkey)
            .slash_authority(authority_pubkey)
            .mint(mint.pubkey())
            .vault(token.pubkey())
            .cooldown_time_seconds(1) // 1 second
            .max_deactivation_basis_points(500) // 5%
            .instruction();

        // When we try to initialize the config with a non-empty token.

        let tx = Transaction::new_signed_with_payer(
            &[create_ix, intialize_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &config],
            context.last_blockhash,
        );

        let err = context
            .banks_client
            .process_transaction(tx)
            .await
            .unwrap_err();

        // Then we expect an error.

        assert_custom_error!(err, StakeError::AmountGreaterThanZero);
    }
}
