#![cfg(feature = "test-sbf")]

mod setup;

use borsh::BorshSerialize;
use paladin_stake_program_client::{
    accounts::{Config, SolStakerStake, ValidatorStake},
    errors::PaladinStakeProgramError,
    instructions::InactivateSolStakerStakeBuilder,
    pdas::find_sol_staker_stake_pda,
    NullableU64,
};
use setup::{
    config::{create_config, create_config_with_args},
    setup,
    sol_staker_stake::SolStakerStakeManager,
    validator_stake::ValidatorStakeManager,
};
use solana_program_test::tokio;
use solana_sdk::{
    account::{Account, AccountSharedData},
    clock::Clock,
    instruction::InstructionError,
    pubkey::Pubkey,
    signature::Signer,
    transaction::Transaction,
};

#[tokio::test]
async fn inactivate_sol_staker_stake() {
    let mut context = setup().await;

    // Given a config account (total amount delegated = 100).

    let config = create_config(&mut context).await;
    // "manually" set the total amount delegated
    let mut account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_delegated = 100;
    // "manually" update the config account data
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config, &account.into());

    // And a validator stake account (total amount staked = 100).

    let validator_stake_manager = ValidatorStakeManager::new(&mut context, &config).await;

    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the stake values
    stake_account.total_staked_token_amount = 100;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    // And a SOL staker stake account (amount staked = 100, deactivating amount = 50).

    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        1_000_000_000,
    )
    .await;

    let mut account = get_account!(context, sol_staker_stake_manager.stake);
    let mut stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the stake values
    stake_account.delegation.amount = 100;
    stake_account.delegation.deactivating_amount = 50;

    let mut timestamp = context
        .banks_client
        .get_sysvar::<Clock>()
        .await
        .unwrap()
        .unix_timestamp as u64;
    timestamp = timestamp.saturating_sub(config_account.cooldown_time_seconds);
    stake_account.delegation.deactivation_timestamp = NullableU64::from(timestamp);

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &account.into());

    // When we move the deactivated amount to inactive (50 tokens).

    let inactivate_ix = InactivateSolStakerStakeBuilder::new()
        .config(config)
        .stake(sol_staker_stake_manager.stake)
        .validator_stake(validator_stake_manager.stake)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[inactivate_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the inactivation should be successful.

    let account = get_account!(context, sol_staker_stake_manager.stake);
    let stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();

    assert_eq!(stake_account.delegation.amount, 50);
    assert_eq!(stake_account.delegation.deactivating_amount, 0);
    assert_eq!(stake_account.delegation.inactive_amount, 50);
    assert!(stake_account
        .delegation
        .deactivation_timestamp
        .value()
        .is_none());

    // And the total staked on the validator stake was updated.

    let account = get_account!(context, validator_stake_manager.stake);
    let validator_stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();

    assert_eq!(validator_stake_account.total_staked_token_amount, 50);

    // And the total delegated on the config was updated.

    let account = get_account!(context, config);
    let config_account = Config::from_bytes(account.data.as_ref()).unwrap();

    assert_eq!(config_account.token_amount_delegated, 50);
}

#[tokio::test]
async fn fail_inactivate_sol_staker_stake_with_no_deactivated_amount() {
    let mut context = setup().await;

    // Given a config account (total amount delegated = 100).

    let config = create_config(&mut context).await;
    // "manually" set the total amount delegated
    let mut account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_delegated = 100;
    // "manually" update the config account data
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config, &account.into());

    // And a validator stake account (total amount staked = 100).

    let validator_stake_manager = ValidatorStakeManager::new(&mut context, &config).await;

    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the stake values
    stake_account.total_staked_token_amount = 100;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    // And a SOL staker stake account (amount staked = 100, deactivating amount = 0).

    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        1_000_000_000,
    )
    .await;

    let mut account = get_account!(context, sol_staker_stake_manager.stake);
    let mut stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the stake values
    stake_account.delegation.amount = 100;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &account.into());

    // When we try to inactivate the stake without any deactivated amount.

    let inactivate_ix = InactivateSolStakerStakeBuilder::new()
        .config(config)
        .stake(sol_staker_stake_manager.stake)
        .validator_stake(validator_stake_manager.stake)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[inactivate_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Then we expect an error.

    assert_custom_error!(err, PaladinStakeProgramError::NoDeactivatedTokens);
}

#[tokio::test]
async fn fail_inactivate_sol_staker_stake_with_wrong_config() {
    let mut context = setup().await;

    // Given a config account (total amount delegated = 100).

    let config = create_config(&mut context).await;
    // "manually" set the total amount delegated
    let mut account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_delegated = 100;
    // "manually" update the config account data
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config, &account.into());

    // And a validator stake account (total amount staked = 100).

    let validator_stake_manager = ValidatorStakeManager::new(&mut context, &config).await;

    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the stake values
    stake_account.total_staked_token_amount = 100;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    // And a SOL staker stake account (amount staked = 100, deactivating amount = 50).

    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        1_000_000_000,
    )
    .await;

    let mut account = get_account!(context, sol_staker_stake_manager.stake);
    let mut stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the stake values
    stake_account.delegation.amount = 100;
    stake_account.delegation.deactivating_amount = 50;

    let mut timestamp = context
        .banks_client
        .get_sysvar::<Clock>()
        .await
        .unwrap()
        .unix_timestamp as u64;
    timestamp = timestamp.saturating_sub(config_account.cooldown_time_seconds);
    stake_account.delegation.deactivation_timestamp = NullableU64::from(timestamp);

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &account.into());

    // And we create a second config.

    let wrong_config = create_config(&mut context).await;

    // When we try to inactivate the stake with the wrong config account.

    let inactivate_ix = InactivateSolStakerStakeBuilder::new()
        .config(wrong_config) // <- wrong config
        .stake(sol_staker_stake_manager.stake)
        .validator_stake(validator_stake_manager.stake)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[inactivate_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Then we expect an error.

    assert_instruction_error!(err, InstructionError::InvalidSeeds);
}

#[tokio::test]
async fn fail_inactivate_sol_stake_stake_with_uninitialized_stake_account() {
    let mut context = setup().await;

    // Given a config and validator stake accounts.

    let config = create_config(&mut context).await;
    let validator_stake_manager = ValidatorStakeManager::new(&mut context, &config).await;

    // And an uninitialized SOL staker stake pda.

    let (sol_staker_stake_pda, _) = find_sol_staker_stake_pda(&Pubkey::new_unique(), &config);

    context.set_account(
        &sol_staker_stake_pda,
        &AccountSharedData::from(Account {
            lamports: 100_000_000,
            data: vec![5; SolStakerStake::LEN],
            owner: paladin_stake_program_client::ID,
            ..Default::default()
        }),
    );

    // When we try to deactivate from an uninitialized stake account.

    let inactivate_ix = InactivateSolStakerStakeBuilder::new()
        .config(config)
        .stake(sol_staker_stake_pda)
        .validator_stake(validator_stake_manager.stake)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[inactivate_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
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
async fn fail_inactivate_sol_staker_stake_with_active_cooldown() {
    let mut context = setup().await;

    // Given a config account (total amount delegated = 100).

    let config = create_config_with_args(
        &mut context,
        10,        /* cooldown 10 seconds */
        500,       /* basis points 5%     */
        1_000_000, /* sync rewards lamports 0.001 SOL */
    )
    .await;
    // "manually" set the total amount delegated
    let mut account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_delegated = 100;
    // "manually" update the config account data
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config, &account.into());

    // And a validator stake account (total amount staked = 100).

    let validator_stake_manager = ValidatorStakeManager::new(&mut context, &config).await;

    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the stake values
    stake_account.total_staked_token_amount = 100;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    // And a SOL staker stake account (amount staked = 100, deactivating amount = 50).

    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        1_000_000_000,
    )
    .await;

    let mut account = get_account!(context, sol_staker_stake_manager.stake);
    let mut stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the stake values
    stake_account.delegation.amount = 100;
    stake_account.delegation.deactivating_amount = 50;

    let timestamp = context
        .banks_client
        .get_sysvar::<Clock>()
        .await
        .unwrap()
        .unix_timestamp as u64;
    stake_account.delegation.deactivation_timestamp = NullableU64::from(timestamp);

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &account.into());

    // When we try to move the deactivated amount to inactive before the end of
    // the cooldown period.

    let inactivate_ix = InactivateSolStakerStakeBuilder::new()
        .config(config)
        .stake(sol_staker_stake_manager.stake)
        .validator_stake(validator_stake_manager.stake)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[inactivate_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Then we expect an error.

    assert_custom_error!(err, PaladinStakeProgramError::ActiveDeactivationCooldown);
}

#[tokio::test]
async fn fail_inactivate_sol_staker_stake_with_wrong_validator_stake() {
    let mut context = setup().await;

    // Given a config account (total amount delegated = 100).

    let config = create_config(&mut context).await;
    // "manually" set the total amount delegated
    let mut account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_delegated = 100;
    // "manually" update the config account data
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config, &account.into());

    // And a validator stake account (total amount staked = 100).

    let validator_stake_manager = ValidatorStakeManager::new(&mut context, &config).await;

    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the stake values
    stake_account.total_staked_token_amount = 100;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    // And a SOL staker stake account (amount staked = 100, deactivating amount = 50).

    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        1_000_000_000,
    )
    .await;

    let mut account = get_account!(context, sol_staker_stake_manager.stake);
    let mut stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the stake values
    stake_account.delegation.amount = 100;
    stake_account.delegation.deactivating_amount = 50;

    let mut timestamp = context
        .banks_client
        .get_sysvar::<Clock>()
        .await
        .unwrap()
        .unix_timestamp as u64;
    timestamp = timestamp.saturating_sub(config_account.cooldown_time_seconds);
    stake_account.delegation.deactivation_timestamp = NullableU64::from(timestamp);

    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &account.into());

    // And we create a second validator stake.

    let wrong_config = create_config(&mut context).await;
    let wrong_validator_stake = ValidatorStakeManager::new(&mut context, &wrong_config)
        .await
        .stake;

    // When we try to inactivate the stake with the wrong validator stake account.

    let inactivate_ix = InactivateSolStakerStakeBuilder::new()
        .config(config)
        .stake(sol_staker_stake_manager.stake)
        .validator_stake(wrong_validator_stake) // <- wrong validator stake
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[inactivate_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Then we expect an error.

    assert_instruction_error!(err, InstructionError::InvalidSeeds);
}
