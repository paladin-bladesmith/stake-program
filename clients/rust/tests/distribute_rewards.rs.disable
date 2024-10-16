#![cfg(feature = "test-sbf")]

mod setup;

use borsh::BorshSerialize;
use paladin_stake_program_client::{accounts::Config, instructions::DistributeRewardsBuilder};
use setup::{config::create_config, REWARDS_PER_TOKEN_SCALING_FACTOR};
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{
    account::Account,
    instruction::InstructionError,
    signature::{Keypair, Signer},
    system_instruction::{self, SystemError},
    transaction::Transaction,
};

#[tokio::test]
async fn distribute_rewards() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config account.

    let config = create_config(&mut context).await;
    // "manually" set the total amount delegated
    let account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 5; // <- 5 tokens

    let updated_config = Account {
        lamports: account.lamports,
        data: config_account.try_to_vec().unwrap(),
        owner: account.owner,
        executable: account.executable,
        rent_epoch: account.rent_epoch,
    };
    context.set_account(&config, &updated_config.into());

    // When we distribute rewards.

    let distribute_rewards_ix = DistributeRewardsBuilder::new()
        .config(config)
        .payer(context.payer.pubkey())
        .amount(5_000_000_000)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[distribute_rewards_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the config account should have the accumulated stake rewards per token updated.

    let account = get_account!(context, config);
    let account_data = account.data.as_ref();
    let config_account = Config::from_bytes(account_data).unwrap();

    assert_eq!(
        config_account.accumulated_stake_rewards_per_token,
        1_000_000_000u128
            // accumulated rewards are stored with a 1e18 scaling factor
            .checked_mul(REWARDS_PER_TOKEN_SCALING_FACTOR)
            .unwrap()
    );
}

#[tokio::test]
async fn distribute_rewards_wrapped() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config account.

    let config = create_config(&mut context).await;
    // "manually" set the total amount delegated and max reward.
    let account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 5; // <- 5 tokens
    config_account.accumulated_stake_rewards_per_token = u128::MAX; // Maximum rate.

    let updated_config = Account {
        lamports: account.lamports,
        data: config_account.try_to_vec().unwrap(),
        owner: account.owner,
        executable: account.executable,
        rent_epoch: account.rent_epoch,
    };
    context.set_account(&config, &updated_config.into());

    // When we distribute rewards.

    let distribute_rewards_ix = DistributeRewardsBuilder::new()
        .config(config)
        .payer(context.payer.pubkey())
        .amount(5_000_000_000)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[distribute_rewards_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the config account should have the accumulated stake rewards per
    // token updated with wrapped math.

    let account = get_account!(context, config);
    let account_data = account.data.as_ref();
    let config_account = Config::from_bytes(account_data).unwrap();

    assert_eq!(
        config_account.accumulated_stake_rewards_per_token,
        1_000_000_000u128
            // accumulated rewards are stored with a 1e18 scaling factor
            .checked_mul(REWARDS_PER_TOKEN_SCALING_FACTOR)
            .and_then(|p| p.checked_sub(1))
            .unwrap()
    );
}

#[tokio::test]
async fn distribute_rewards_with_no_staked_tokens() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config account.

    let config = create_config(&mut context).await;
    let account = get_account!(context, config);
    let lamports = account.lamports;

    // When we distribute rewards with no staked tokens.

    let distribute_rewards_ix = DistributeRewardsBuilder::new()
        .config(config)
        .payer(context.payer.pubkey())
        .amount(5_000_000_000)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[distribute_rewards_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the config account should not have the accumulated stake rewards per token updated
    // and the lamports are updated.

    let account = get_account!(context, config);
    let account_data = account.data.as_ref();
    let config_account = Config::from_bytes(account_data).unwrap();

    assert_eq!(config_account.accumulated_stake_rewards_per_token, 0u128);
    assert_eq!(account.lamports, lamports.saturating_add(5_000_000_000));
}

#[tokio::test]
async fn fail_distribute_rewards_with_uninitialized_config() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given an uninitialized config account.

    let uninitialized_config = Keypair::new();

    let create_config_ix = system_instruction::create_account(
        &context.payer.pubkey(),
        &uninitialized_config.pubkey(),
        context
            .banks_client
            .get_rent()
            .await
            .unwrap()
            .minimum_balance(Config::LEN),
        Config::LEN as u64,
        &paladin_stake_program_client::ID,
    );

    // When we try distribute rewards to an uninitialized config account.

    let distribute_rewards_ix = DistributeRewardsBuilder::new()
        .config(uninitialized_config.pubkey())
        .payer(context.payer.pubkey())
        .amount(5_000_000_000)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[create_config_ix, distribute_rewards_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &uninitialized_config],
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

#[tokio::test]
async fn fail_distribute_rewards_with_payer_insufficient_funds() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config account and a payer with no funds.

    let config = create_config(&mut context).await;
    // "manually" set the total amount delegated
    let account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 5; // <- 5 tokens

    let updated_config = Account {
        lamports: account.lamports,
        data: config_account.try_to_vec().unwrap(),
        owner: account.owner,
        executable: account.executable,
        rent_epoch: account.rent_epoch,
    };
    context.set_account(&config, &updated_config.into());

    let payer = Keypair::new();

    // When we try to distribute rewards with a payer with no funds.

    let distribute_rewards_ix = DistributeRewardsBuilder::new()
        .config(config)
        .payer(payer.pubkey())
        .amount(5_000_000_000)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[distribute_rewards_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &payer],
        context.last_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Then we expect an error and the config account should not have the accumulated stake rewards
    // per token updated.

    assert_custom_error!(err, SystemError::ResultWithNegativeLamports);

    let account = get_account!(context, config);
    let account_data = account.data.as_ref();
    let config_account = Config::from_bytes(account_data).unwrap();

    assert_eq!(config_account.accumulated_stake_rewards_per_token, 0u128);
}
