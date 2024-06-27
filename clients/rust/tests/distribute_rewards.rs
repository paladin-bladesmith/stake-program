#![cfg(feature = "test-sbf")]

mod setup;

use paladin_stake::{
    accounts::Config, instructions::DistributeRewardsBuilder,
    instructions::InitializeConfigBuilder, pdas::find_vault_pda,
};
use setup::{airdrop, create_mint, create_token, MINT_EXTENSIONS, TOKEN_ACCOUNT_EXTENSIONS};
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{
    instruction::InstructionError,
    native_token::sol_to_lamports,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};

mod initialize_config {

    use super::*;

    #[tokio::test]
    async fn distribute_rewards_with_payer() {
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

        // And we initialize the config.

        let tx = Transaction::new_signed_with_payer(
            &[create_ix, initialize_ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &config],
            context.last_blockhash,
        );
        context.banks_client.process_transaction(tx).await.unwrap();

        let account = get_account!(context, config.pubkey());
        let config_account = Config::from_bytes(account.data.as_ref()).unwrap();

        assert_eq!(config_account.total_stake_rewards, 0);

        // When we distribute rewards.

        let amount = sol_to_lamports(10f64);
        let payer = Keypair::new();

        airdrop(&mut context, &payer.pubkey(), amount)
            .await
            .unwrap();

        let ix = DistributeRewardsBuilder::new()
            .payer(payer.pubkey())
            .config(config.pubkey())
            .args(amount)
            .instruction();

        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &payer],
            context.last_blockhash,
        );
        context.banks_client.process_transaction(tx).await.unwrap();

        // Then the total rewards are updated.

        let account = get_account!(context, config.pubkey());
        let config_account = Config::from_bytes(account.data.as_ref()).unwrap();

        assert_eq!(config_account.total_stake_rewards, amount);
    }

    #[tokio::test]
    async fn fail_distribute_rewards_with_uninitialized_config() {
        let mut context = ProgramTest::new("stake_program", paladin_stake::ID, None)
            .start_with_context()
            .await;

        // When we distribute rewards to an uninitialized config account.

        let config = Keypair::new();

        let amount = sol_to_lamports(10f64);
        let payer = Keypair::new();

        airdrop(&mut context, &payer.pubkey(), amount)
            .await
            .unwrap();

        let ix = DistributeRewardsBuilder::new()
            .payer(payer.pubkey())
            .config(config.pubkey())
            .args(amount)
            .instruction();

        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &payer],
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
}
