#![cfg(feature = "test-sbf")]

mod setup;

use paladin_stake::{
    accounts::Config,
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
    use super::*;

    #[tokio::test]
    async fn set_authority_on_config() {
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
            .vault_token(token.pubkey())
            .cooldown_time_seconds(1) // 1 second
            .max_deactivation_basis_points(500) // 5%
            .instruction();

        // And we create a config.

        let tx = Transaction::new_signed_with_payer(
            &[create_ix, initialize_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &config],
            context.last_blockhash,
        );
        context.banks_client.process_transaction(tx).await.unwrap();

        let account = get_account!(context, config.pubkey());
        let config_account = Config::from_bytes(account.data.as_ref()).unwrap();
        assert_eq!(config_account.authority, authority_pubkey);

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
        assert_eq!(config_account.authority, new_authority.pubkey());
    }
}
