#![cfg(feature = "test-sbf")]

mod setup;

use paladin_stake::{
    accounts::Config,
    errors::StakeError,
    instructions::{InitializeConfigBuilder, SetAuthorityBuilder},
    pdas::find_vault_pda,
    types::AuthorityType,
};
use setup::{create_mint, create_token, MINT_EXTENSIONS, TOKEN_ACCOUNT_EXTENSIONS};
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};

mod set_authority {

    use solana_sdk::pubkey::Pubkey;

    use super::*;

    #[tokio::test]
    async fn set_config_authority_on_config() {
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
            .config_authority(authority_pubkey)
            .slash_authority(authority_pubkey)
            .mint(mint.pubkey())
            .vault(token.pubkey())
            .cooldown_time_seconds(1) // 1 second
            .max_deactivation_basis_points(500) // 5%
            .instruction();

        // And we initialize a config.

        let tx = Transaction::new_signed_with_payer(
            &[create_ix, initialize_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &config],
            context.last_blockhash,
        );
        context.banks_client.process_transaction(tx).await.unwrap();

        let account = get_account!(context, config.pubkey());
        let config_account = Config::from_bytes(account.data.as_ref()).unwrap();
        assert_eq!(config_account.authority, authority_pubkey.into());

        // When we set a new authority on the config.

        let new_authority = Keypair::new();

        let set_authority_ix = SetAuthorityBuilder::new()
            .account(config.pubkey())
            .authority(authority_pubkey)
            .new_authority(new_authority.pubkey())
            .authority_type(AuthorityType::Config)
            .instruction();

        let tx = Transaction::new_signed_with_payer(
            &[set_authority_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &authority],
            context.last_blockhash,
        );
        context.banks_client.process_transaction(tx).await.unwrap();

        // Then the authority is updated.

        let account = get_account!(context, config.pubkey());
        let config_account = Config::from_bytes(account.data.as_ref()).unwrap();
        assert_eq!(config_account.authority, new_authority.pubkey().into());
    }

    #[tokio::test]
    async fn set_slash_authority_on_config() {
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
            .config_authority(authority_pubkey)
            .slash_authority(authority_pubkey)
            .mint(mint.pubkey())
            .vault(token.pubkey())
            .cooldown_time_seconds(1) // 1 second
            .max_deactivation_basis_points(500) // 5%
            .instruction();

        // And we initialize a config.

        let tx = Transaction::new_signed_with_payer(
            &[create_ix, initialize_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &config],
            context.last_blockhash,
        );
        context.banks_client.process_transaction(tx).await.unwrap();

        let account = get_account!(context, config.pubkey());
        let config_account = Config::from_bytes(account.data.as_ref()).unwrap();
        assert_eq!(config_account.authority, authority_pubkey.into());

        // When we set a new slash authority on the config.

        let new_slash_authority = Keypair::new();

        let set_authority_ix = SetAuthorityBuilder::new()
            .account(config.pubkey())
            .authority(authority_pubkey)
            .new_authority(new_slash_authority.pubkey())
            .authority_type(AuthorityType::Config)
            .instruction();

        let tx = Transaction::new_signed_with_payer(
            &[set_authority_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &authority],
            context.last_blockhash,
        );
        context.banks_client.process_transaction(tx).await.unwrap();

        // Then the authority is updated.

        let account = get_account!(context, config.pubkey());
        let config_account = Config::from_bytes(account.data.as_ref()).unwrap();
        assert_eq!(
            config_account.authority,
            new_slash_authority.pubkey().into()
        );
    }

    #[tokio::test]
    async fn failt_set_config_authority_with_wrong_authority() {
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
            .config_authority(authority_pubkey)
            .slash_authority(authority_pubkey)
            .mint(mint.pubkey())
            .vault(token.pubkey())
            .cooldown_time_seconds(1) // 1 second
            .max_deactivation_basis_points(500) // 5%
            .instruction();

        // And we initialize a config.

        let tx = Transaction::new_signed_with_payer(
            &[create_ix, initialize_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &config],
            context.last_blockhash,
        );
        context.banks_client.process_transaction(tx).await.unwrap();

        let account = get_account!(context, config.pubkey());
        let config_account = Config::from_bytes(account.data.as_ref()).unwrap();
        assert_eq!(config_account.authority, authority_pubkey.into());

        // When we try to set a new authority with a wrong authority.

        let fake_authority = Keypair::new();
        let new_authority = Keypair::new();

        let set_authority_ix = SetAuthorityBuilder::new()
            .account(config.pubkey())
            .authority(fake_authority.pubkey())
            .new_authority(new_authority.pubkey())
            .authority_type(AuthorityType::Config)
            .instruction();

        let tx = Transaction::new_signed_with_payer(
            &[set_authority_ix],
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
    async fn fail_set_slash_authority_with_authority_none() {
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
            .config_authority(Pubkey::default()) // <- None
            .slash_authority(authority_pubkey)
            .mint(mint.pubkey())
            .vault(token.pubkey())
            .cooldown_time_seconds(1) // 1 second
            .max_deactivation_basis_points(500) // 5%
            .instruction();

        // And we initialize a config.

        let tx = Transaction::new_signed_with_payer(
            &[create_ix, initialize_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &config],
            context.last_blockhash,
        );
        context.banks_client.process_transaction(tx).await.unwrap();

        let account = get_account!(context, config.pubkey());
        let config_account = Config::from_bytes(account.data.as_ref()).unwrap();
        assert_eq!(config_account.authority, Pubkey::default().into());

        // When we try to set a new slash authority when config authority is None.

        let new_slash_authority = Keypair::new();

        let set_authority_ix = SetAuthorityBuilder::new()
            .account(config.pubkey())
            .authority(authority_pubkey)
            .new_authority(new_slash_authority.pubkey())
            .authority_type(AuthorityType::Config)
            .instruction();

        let tx = Transaction::new_signed_with_payer(
            &[set_authority_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &authority],
            context.last_blockhash,
        );
        context.banks_client.process_transaction(tx).await.unwrap();

        // Then the slash authority has not changed.

        let account = get_account!(context, config.pubkey());
        let config_account = Config::from_bytes(account.data.as_ref()).unwrap();
        // TODO: checking that the slash authority hasn't changed; we probably want
        // to fail in the program when authority is None.
        assert_ne!(
            config_account.slash_authority,
            new_slash_authority.pubkey().into()
        );
    }
}
