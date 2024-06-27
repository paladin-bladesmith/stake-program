#![cfg(feature = "test-sbf")]

mod setup;

use paladin_stake::{
    accounts::Config,
    errors::StakeError,
    instructions::{InitializeConfigBuilder, UpdateConfigBuilder},
    pdas::find_vault_pda,
    types::ConfigField,
};
use setup::{create_mint, create_token, MINT_EXTENSIONS, TOKEN_ACCOUNT_EXTENSIONS};
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{
    instruction::InstructionError,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};

mod update_config {

    use super::*;

    #[tokio::test]
    async fn update_config() {
        let mut context = ProgramTest::new("stake_program", paladin_stake::ID, None)
            .start_with_context()
            .await;

        // Given an empty config account and a mint.

        let config = Keypair::new();
        let authority = Keypair::new();

        let mint = Keypair::new();
        create_mint(
            &mut context,
            &mint,
            &authority.pubkey(),
            Some(&authority.pubkey()),
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

        // And we create a config.

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
            .config_authority(authority.pubkey())
            .slash_authority(authority.pubkey())
            .mint(mint.pubkey())
            .vault(token.pubkey())
            .cooldown_time_seconds(1) // 1 second
            .max_deactivation_basis_points(500) // 5%
            .instruction();

        let tx = Transaction::new_signed_with_payer(
            &[create_ix, initialize_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &config],
            context.last_blockhash,
        );
        context.banks_client.process_transaction(tx).await.unwrap();

        let account = get_account!(context, config.pubkey());
        let config_account = Config::from_bytes(account.data.as_ref()).unwrap();
        assert_eq!(config_account.cooldown_time_seconds, 1);

        // When we update the config.

        let ix = UpdateConfigBuilder::new()
            .config(config.pubkey())
            .config_authority(authority.pubkey())
            .config_field(ConfigField::CooldownTimeSeconds(10)) // 10 seconds
            .instruction();

        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &authority],
            context.last_blockhash,
        );
        context.banks_client.process_transaction(tx).await.unwrap();

        // Then the config was updated.

        let account = get_account!(context, config.pubkey());
        let config_account = Config::from_bytes(account.data.as_ref()).unwrap();
        assert_eq!(config_account.cooldown_time_seconds, 10);
    }

    #[tokio::test]
    async fn fail_update_config_with_wrong_authority() {
        let mut context = ProgramTest::new("stake_program", paladin_stake::ID, None)
            .start_with_context()
            .await;

        // Given an empty config account and a mint.

        let config = Keypair::new();
        let authority = Keypair::new();

        let mint = Keypair::new();
        create_mint(
            &mut context,
            &mint,
            &authority.pubkey(),
            Some(&authority.pubkey()),
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

        // And we create a config.

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
            .config_authority(authority.pubkey())
            .slash_authority(authority.pubkey())
            .mint(mint.pubkey())
            .vault(token.pubkey())
            .cooldown_time_seconds(1) // 1 second
            .max_deactivation_basis_points(500) // 5%
            .instruction();

        let tx = Transaction::new_signed_with_payer(
            &[create_ix, initialize_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &config],
            context.last_blockhash,
        );
        context.banks_client.process_transaction(tx).await.unwrap();

        let account = get_account!(context, config.pubkey());
        let config_account = Config::from_bytes(account.data.as_ref()).unwrap();
        assert_eq!(config_account.cooldown_time_seconds, 1);

        // When we try to update the config with a wrong authority.
        let fake_authority = Keypair::new();

        let ix = UpdateConfigBuilder::new()
            .config(config.pubkey())
            .config_authority(fake_authority.pubkey())
            .config_field(ConfigField::CooldownTimeSeconds(10)) // 10 seconds
            .instruction();

        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &fake_authority],
            context.last_blockhash,
        );

        let err = context
            .banks_client
            .process_transaction(tx)
            .await
            .unwrap_err();

        // Then we expect an error.

        assert_custom_error!(err, StakeError::InvalidAuthority);
    }

    #[tokio::test]
    async fn fail_update_config_non_existing() {
        let mut context = ProgramTest::new("stake_program", paladin_stake::ID, None)
            .start_with_context()
            .await;

        // Given an non-existing config account and a mint.

        let config = Keypair::new();
        let authority = Keypair::new();

        let mint = Keypair::new();
        create_mint(
            &mut context,
            &mint,
            &authority.pubkey(),
            Some(&authority.pubkey()),
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

        // And we try to update a non-existing config.

        let ix = UpdateConfigBuilder::new()
            .config(config.pubkey())
            .config_authority(authority.pubkey())
            .config_field(ConfigField::CooldownTimeSeconds(10))
            .instruction();

        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &authority],
            context.last_blockhash,
        );

        let err = context
            .banks_client
            .process_transaction(tx)
            .await
            .unwrap_err();

        // Then we expect an error.

        assert_instruction_error!(err, InstructionError::InvalidAccountOwner);
    }

    #[tokio::test]
    async fn fail_update_with_uninitialized_config() {
        let mut context = ProgramTest::new("stake_program", paladin_stake::ID, None)
            .start_with_context()
            .await;

        // Given an empty config account and a mint.

        let config = Keypair::new();
        let authority = Keypair::new();

        let mint = Keypair::new();
        create_mint(
            &mut context,
            &mint,
            &authority.pubkey(),
            Some(&authority.pubkey()),
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

        // When we try to update the empty config.

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

        let update_ix = UpdateConfigBuilder::new()
            .config(config.pubkey())
            .config_authority(authority.pubkey())
            .config_field(ConfigField::CooldownTimeSeconds(10))
            .instruction();

        let tx = Transaction::new_signed_with_payer(
            &[create_ix, update_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &config, &authority],
            context.last_blockhash,
        );

        let err = context
            .banks_client
            .process_transaction(tx)
            .await
            .unwrap_err();

        // Then we expect an error.

        assert_instruction_error!(err, InstructionError::UninitializedAccount);
    }
}
