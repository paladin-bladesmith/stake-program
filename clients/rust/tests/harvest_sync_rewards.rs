#![cfg(feature = "test-sbf")]

mod setup;

use borsh::BorshSerialize;
use paladin_stake_program_client::{
    accounts::{Config, SolStakerStake, ValidatorStake},
    errors::PaladinStakeProgramError,
    instructions::HarvestSyncRewardsBuilder,
};
use setup::{
    calculate_stake_rewards_per_token,
    config::ConfigManager,
    new_program_test,
    sol_staker_stake::SolStakerStakeManager,
    stake::{create_stake_account, deactivate_stake_account},
    validator_stake::ValidatorStakeManager,
    vote::add_vote_account,
};
use solana_program_test::tokio;
use solana_sdk::{
    account::{Account, AccountSharedData},
    instruction::InstructionError,
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    stake::state::{Authorized, Lockup},
    transaction::Transaction,
};

/// The rounding error tollerance for the rewards per token calculation.
///
/// This is necessary since the rewards per token calculation might truncated the result and
/// the tests are comparing values derived from this, so we need to allow for a small error.
const REWARD_PER_TOKEN_ROUNDING_ERROR: u128 = 1;

#[tokio::test]
async fn harvest_sync_rewards() {
    let mut program_test = new_program_test();
    let vote = add_vote_account(
        &mut program_test,
        &Pubkey::new_unique(),
        &Pubkey::new_unique(),
    );
    let mut context = program_test.start_with_context().await;
    let slot = context.genesis_config().epoch_schedule.first_normal_slot + 1;
    context.warp_to_slot(slot).unwrap();

    // Given a config, validator stake and sol staker stake accounts with 1 SOL staked.

    // default sync_rewards_lamports = 1_000_000 (0.001 SOL)
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new_with_vote(&mut context, &config_manager.config, vote).await;
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        1_000_000_000, // 1 SOL staked
    )
    .await;

    // And there is 1 SOL for stake rewards on the config.

    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_delegated = 1_300_000_000;
    config_account.accumulated_stake_rewards_per_token =
        calculate_stake_rewards_per_token(1_300_000_000, 1_300_000_000);

    account.lamports += 1_300_000_000; // 1 SOL
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    // And the SOL staker stake has 1_300_000_000 tokens staked.

    let mut account = get_account!(context, sol_staker_stake_manager.stake);
    let mut stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the staked values:
    //   - delegation amount = 1_300_000_000
    //   - lamports amount = 1_000_000_000
    stake_account.delegation.amount = 1_300_000_000;
    stake_account.lamports_amount = 1_000_000_000;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &account.into());

    // And the SOL staker stake and validator stake accounts are correctly synced.

    let account = get_account!(context, sol_staker_stake_manager.stake);
    let stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();

    assert_eq!(stake_account.lamports_amount, 1_000_000_000);

    let account = get_account!(context, validator_stake_manager.stake);
    let validator_stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        validator_stake_account.total_staked_lamports_amount,
        1_000_000_000
    );

    // And we deactivate the stake.

    deactivate_stake_account(
        &mut context,
        &stake_account.sol_stake,
        &sol_staker_stake_manager.authority,
    )
    .await;

    // When we harvest rewards for syncing the SOL stake after deactivating the SOL stake.

    let destination = Pubkey::new_unique();
    context.set_account(
        &destination,
        &AccountSharedData::from(Account {
            // amount to cover the account rent
            lamports: 100_000_000,
            ..Default::default()
        }),
    );

    let sync_ix = HarvestSyncRewardsBuilder::new()
        .config(config_manager.config)
        .sol_staker_stake(sol_staker_stake_manager.stake)
        .validator_stake(validator_stake_manager.stake)
        .sol_stake(sol_staker_stake_manager.sol_stake)
        .destination(destination)
        .sol_stake_view_program(paladin_sol_stake_view_program_client::ID)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[sync_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the destination account has the sync rewards.

    let account = get_account!(context, destination);
    assert_eq!(account.lamports, 100_000_000 + 1_000_000); // rent + rewards

    // Then the SOL amounts are correctly synced (0 SOL staked).

    let account = get_account!(context, validator_stake_manager.stake);
    let validator_stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(validator_stake_account.total_staked_lamports_amount, 0);

    let account = get_account!(context, sol_staker_stake_manager.stake);
    let stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();

    assert_eq!(stake_account.lamports_amount, 0);

    // And the last seen stake rewards per token on the SOL staker stake account
    // was updated correctly.

    let last_seen_stake_rewards_per_token =
        stake_account.delegation.last_seen_stake_rewards_per_token;

    let account = get_account!(context, config_manager.config);
    let config_account = Config::from_bytes(account.data.as_ref()).unwrap();

    assert!(config_account.accumulated_stake_rewards_per_token > last_seen_stake_rewards_per_token);
    assert!(last_seen_stake_rewards_per_token > 0);

    // Expected rewards per token:
    //
    //  - SOL staker stake's token amount: 1_300_000_000
    //  - total stake rewards available: 1_300_000_000
    //  - rewards remanining: 1_300_000_000 - 1_000_000 = 1_299_000_000
    //    (after the searcher rewards are taken)
    //
    //  On the SOL staker stake account:
    //  - last seen rewards per token: 1_000_000 / 1_300_000_000
    //
    //  Effective rewards per token for SOL stake harvest:
    //  - rewards per token for SOL stake harvest: 1_299_000_000 / 1_300_000_000
    assert_eq!(
        last_seen_stake_rewards_per_token,
        calculate_stake_rewards_per_token(1_000_000, 1_300_000_000)
    );

    let difference = config_account
        .accumulated_stake_rewards_per_token
        .checked_sub(last_seen_stake_rewards_per_token)
        .and_then(|rewards| {
            rewards.checked_sub(calculate_stake_rewards_per_token(
                1_299_000_000,
                1_300_000_000,
            ))
        })
        .unwrap();

    assert!(difference <= REWARD_PER_TOKEN_ROUNDING_ERROR);
}

#[tokio::test]
async fn harvest_sync_rewards_without_rewards() {
    let mut program_test = new_program_test();
    let vote = add_vote_account(
        &mut program_test,
        &Pubkey::new_unique(),
        &Pubkey::new_unique(),
    );
    let mut context = program_test.start_with_context().await;
    let slot = context.genesis_config().epoch_schedule.first_normal_slot + 1;
    context.warp_to_slot(slot).unwrap();

    // Given a config, validator stake and sol staker stake accounts with 1 SOL staked.

    // default sync_rewards_lamports = 1_000_000 (0.001 SOL)
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new_with_vote(&mut context, &config_manager.config, vote).await;
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        1_000_000_000, // 1 SOL staked
    )
    .await;

    // And there is 1.3 SOL for stake rewards on the config.

    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_delegated = 1_300_000_000;
    config_account.accumulated_stake_rewards_per_token =
        calculate_stake_rewards_per_token(1_300_000_000, 1_300_000_000);

    account.lamports += 1_300_000_000; // 1.3 SOL
    let expected_config_lamports = account.lamports;

    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    // And the SOL staker stake and validator stake accounts are correctly synced.

    let account = get_account!(context, sol_staker_stake_manager.stake);
    let stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();

    assert_eq!(stake_account.lamports_amount, 1_000_000_000);

    let account = get_account!(context, validator_stake_manager.stake);
    let validator_stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        validator_stake_account.total_staked_lamports_amount,
        1_000_000_000
    );

    // And we deactivate the stake.

    deactivate_stake_account(
        &mut context,
        &stake_account.sol_stake,
        &sol_staker_stake_manager.authority,
    )
    .await;

    // When we harvest rewards for syncing the SOL stake after deactivating the SOL stake
    // on a SOL staker stake account with no rewards.

    let destination = Pubkey::new_unique();
    context.set_account(
        &destination,
        &AccountSharedData::from(Account {
            // amount to cover the account rent
            lamports: 100_000_000,
            ..Default::default()
        }),
    );

    let sync_ix = HarvestSyncRewardsBuilder::new()
        .config(config_manager.config)
        .sol_staker_stake(sol_staker_stake_manager.stake)
        .validator_stake(validator_stake_manager.stake)
        .sol_stake(sol_staker_stake_manager.sol_stake)
        .destination(destination)
        .sol_stake_view_program(paladin_sol_stake_view_program_client::ID)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[sync_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the destination account does not have the sync rewards.

    let account = get_account!(context, destination);
    assert_eq!(account.lamports, 100_000_000); // only rent

    // And the SOL amounts are correctly synced (0 SOL staked).

    let account = get_account!(context, validator_stake_manager.stake);
    let validator_stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();

    assert_eq!(validator_stake_account.total_staked_lamports_amount, 0);

    let account = get_account!(context, sol_staker_stake_manager.stake);
    let stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();

    assert_eq!(stake_account.lamports_amount, 0);
    assert_eq!(
        stake_account.delegation.last_seen_stake_rewards_per_token,
        0
    );

    // And the config lamports amount is the same.

    let account = get_account!(context, config_manager.config);
    assert_eq!(account.lamports, expected_config_lamports);
}

#[tokio::test]
async fn harvest_sync_rewards_with_closed_sol_stake_account() {
    let mut program_test = new_program_test();
    let vote = add_vote_account(
        &mut program_test,
        &Pubkey::new_unique(),
        &Pubkey::new_unique(),
    );
    let mut context = program_test.start_with_context().await;
    let slot = context.genesis_config().epoch_schedule.first_normal_slot + 1;
    context.warp_to_slot(slot).unwrap();

    // Given a config, validator stake and sol staker stake accounts with 1 SOL staked.

    // default sync_rewards_lamports = 1_000_000 (0.001 SOL)
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new_with_vote(&mut context, &config_manager.config, vote).await;
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        1_000_000_000, // 1 SOL staked
    )
    .await;

    // And there is 1 SOL for stake rewards on the config.

    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_delegated = 1_300_000_000;
    config_account.accumulated_stake_rewards_per_token =
        calculate_stake_rewards_per_token(1_300_000_000, 1_300_000_000);

    account.lamports += 1_300_000_000; // 1 SOL
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    // And the SOL staker stake has 1_300_000_000 tokens staked.

    let mut account = get_account!(context, sol_staker_stake_manager.stake);
    let mut stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the staked values:
    //   - delegation amount = 1_300_000_000
    //   - lamports amount = 1_000_000_000
    stake_account.delegation.amount = 1_300_000_000;
    stake_account.lamports_amount = 1_000_000_000;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &account.into());

    // And the SOL staker stake and validator stake accounts are correctly synced.

    let account = get_account!(context, sol_staker_stake_manager.stake);
    let stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();

    assert_eq!(stake_account.lamports_amount, 1_000_000_000);

    let account = get_account!(context, validator_stake_manager.stake);
    let validator_stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        validator_stake_account.total_staked_lamports_amount,
        1_000_000_000
    );

    // And we close the SOL stake account.

    context.set_account(
        &stake_account.sol_stake,
        &AccountSharedData::from(Account {
            data: vec![],
            lamports: 0,
            owner: solana_sdk::stake::program::ID,
            ..Default::default()
        }),
    );

    // When we harvest rewards for syncing the SOL stake after closing the SOL stake account.

    let destination = Pubkey::new_unique();
    context.set_account(
        &destination,
        &AccountSharedData::from(Account {
            // amount to cover the account rent
            lamports: 100_000_000,
            ..Default::default()
        }),
    );

    let sync_ix = HarvestSyncRewardsBuilder::new()
        .config(config_manager.config)
        .sol_staker_stake(sol_staker_stake_manager.stake)
        .validator_stake(validator_stake_manager.stake)
        .sol_stake(sol_staker_stake_manager.sol_stake)
        .destination(destination)
        .sol_stake_view_program(paladin_sol_stake_view_program_client::ID)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[sync_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the destination account has the sync rewards.

    let account = get_account!(context, destination);
    assert_eq!(account.lamports, 100_000_000 + 1_000_000); // rent + rewards

    // Then the SOL amounts are correctly synced (0 SOL staked).

    let account = get_account!(context, validator_stake_manager.stake);
    let validator_stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(validator_stake_account.total_staked_lamports_amount, 0);

    let account = get_account!(context, sol_staker_stake_manager.stake);
    let stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();

    assert_eq!(stake_account.lamports_amount, 0);

    // And the last seen stake rewards per token on the SOL staker stake account
    // was updated correctly.

    let last_seen_stake_rewards_per_token =
        stake_account.delegation.last_seen_stake_rewards_per_token;

    let account = get_account!(context, config_manager.config);
    let config_account = Config::from_bytes(account.data.as_ref()).unwrap();

    assert!(config_account.accumulated_stake_rewards_per_token > last_seen_stake_rewards_per_token);
    assert!(last_seen_stake_rewards_per_token > 0);

    // Expected rewards per token:
    //
    //  - SOL staker stake's token amount: 1_300_000_000
    //  - total stake rewards available: 1_300_000_000
    //  - rewards remanining: 1_300_000_000 - 1_000_000 = 1_299_000_000
    //    (after the searcher rewards are taken)
    //
    //  On the SOL staker stake account:
    //  - last seen rewards per token: 1_000_000 / 1_300_000_000
    //
    //  Effective rewards per token for SOL stake harvest:
    //  - rewards per token for SOL stake harvest: 1_299_000_000 / 1_300_000_000
    assert_eq!(
        last_seen_stake_rewards_per_token,
        calculate_stake_rewards_per_token(1_000_000, 1_300_000_000)
    );

    let difference = config_account
        .accumulated_stake_rewards_per_token
        .checked_sub(last_seen_stake_rewards_per_token)
        .and_then(|rewards| {
            rewards.checked_sub(calculate_stake_rewards_per_token(
                1_299_000_000,
                1_300_000_000,
            ))
        })
        .unwrap();

    assert!(difference <= REWARD_PER_TOKEN_ROUNDING_ERROR);
}

#[tokio::test]
async fn harvest_sync_rewards_with_capped_sync_rewards() {
    let mut program_test = new_program_test();
    let vote = add_vote_account(
        &mut program_test,
        &Pubkey::new_unique(),
        &Pubkey::new_unique(),
    );
    let mut context = program_test.start_with_context().await;
    let slot = context.genesis_config().epoch_schedule.first_normal_slot + 1;
    context.warp_to_slot(slot).unwrap();

    // Given a config, validator stake and sol staker stake accounts with 1 SOL staked.

    // sync_rewards_lamports = 1_300_000_001
    let config_manager = ConfigManager::with_args(&mut context, 1, 500, 1_300_000_001).await;
    let validator_stake_manager =
        ValidatorStakeManager::new_with_vote(&mut context, &config_manager.config, vote).await;
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        1_000_000_000, // 1 SOL staked
    )
    .await;

    // And there is 1_300_000_000 lamports for stake rewards on the config.

    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the total amount delegated
    config_account.token_amount_delegated = 1_300_000_000;
    // add 200_000_000 (0.2) to the accumulated stake rewards per token to simulate
    // a previous reward
    config_account.accumulated_stake_rewards_per_token =
        calculate_stake_rewards_per_token(1_300_000_000, 1_300_000_000) + 200_000_000;

    account.lamports += 1_300_000_000;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    // And the SOL staker stake has 1_300_000_000 tokens staked.

    let mut account = get_account!(context, sol_staker_stake_manager.stake);
    let mut stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the staked values:
    //   - delegation amount = 1_300_000_000
    //   - lamports amount = 1_000_000_000
    stake_account.delegation.amount = 1_300_000_000;
    // set the last seen stake rewards per token to 200_000_000
    stake_account.delegation.last_seen_stake_rewards_per_token = 200_000_000;
    stake_account.lamports_amount = 1_000_000_000;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &account.into());

    // And the SOL staker stake and validator stake accounts are correctly synced.

    let account = get_account!(context, sol_staker_stake_manager.stake);
    let stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();

    assert_eq!(stake_account.lamports_amount, 1_000_000_000);

    let account = get_account!(context, validator_stake_manager.stake);
    let validator_stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        validator_stake_account.total_staked_lamports_amount,
        1_000_000_000
    );

    // And we deactivate the stake.

    deactivate_stake_account(
        &mut context,
        &stake_account.sol_stake,
        &sol_staker_stake_manager.authority,
    )
    .await;

    // When we harvest rewards for syncing the SOL stake after deactivating the SOL stake.

    let destination = Pubkey::new_unique();
    context.set_account(
        &destination,
        &AccountSharedData::from(Account {
            // amount to cover the account rent
            lamports: 100_000_000,
            ..Default::default()
        }),
    );

    let sync_ix = HarvestSyncRewardsBuilder::new()
        .config(config_manager.config)
        .sol_staker_stake(sol_staker_stake_manager.stake)
        .validator_stake(validator_stake_manager.stake)
        .sol_stake(sol_staker_stake_manager.sol_stake)
        .destination(destination)
        .sol_stake_view_program(paladin_sol_stake_view_program_client::ID)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[sync_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the destination account has the sync rewards.

    let account = get_account!(context, destination);
    // rewards capped at 1_300_000_000
    assert_eq!(account.lamports, 100_000_000 + 1_300_000_000);

    // Then the SOL amounts are correctly synced (0 SOL staked).

    let account = get_account!(context, validator_stake_manager.stake);
    let validator_stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(validator_stake_account.total_staked_lamports_amount, 0);

    let account = get_account!(context, sol_staker_stake_manager.stake);
    let stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();

    assert_eq!(stake_account.lamports_amount, 0);

    // And the last seen stake rewards per token on the SOL staker stake account
    // was updated correctly.

    let account = get_account!(context, config_manager.config);
    let config_account = Config::from_bytes(account.data.as_ref()).unwrap();

    // they should be equal since all rewards were harvested as part of the sync rewards
    assert_eq!(
        config_account.accumulated_stake_rewards_per_token,
        stake_account.delegation.last_seen_stake_rewards_per_token
    );
}

#[tokio::test]
async fn fail_harvest_sync_rewards_with_wrong_sol_stake_account() {
    let mut program_test = new_program_test();
    let vote = add_vote_account(
        &mut program_test,
        &Pubkey::new_unique(),
        &Pubkey::new_unique(),
    );
    let mut context = program_test.start_with_context().await;
    let slot = context.genesis_config().epoch_schedule.first_normal_slot + 1;
    context.warp_to_slot(slot).unwrap();

    // Given a config, validator stake and sol staker stake accounts with 1 SOL staked.

    // default sync_rewards_lamports = 1_000_000 (0.001 SOL)
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new_with_vote(&mut context, &config_manager.config, vote).await;
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        1_000_000_000, // 1 SOL staked
    )
    .await;

    // And we create another SOL stake account.

    let another_sol_stake = Keypair::new();
    let authority = Keypair::new();

    create_stake_account(
        &mut context,
        &another_sol_stake,
        &Authorized::auto(&authority.pubkey()),
        &Lockup::default(),
        0, // <- no SOL staked
    )
    .await;

    // When we try to harvest rewards for syncing the SOL stake with the wrong SOL stake account.

    let destination = Pubkey::new_unique();
    context.set_account(
        &destination,
        &AccountSharedData::from(Account {
            // amount to cover the account rent
            lamports: 100_000_000,
            ..Default::default()
        }),
    );

    let sync_ix = HarvestSyncRewardsBuilder::new()
        .config(config_manager.config)
        .sol_staker_stake(sol_staker_stake_manager.stake)
        .validator_stake(validator_stake_manager.stake)
        .sol_stake(another_sol_stake.pubkey()) // <- wrong SOL stake account
        .destination(destination)
        .sol_stake_view_program(paladin_sol_stake_view_program_client::ID)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[sync_ix],
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

    assert_custom_error!(err, PaladinStakeProgramError::IncorrectSolStakeAccount);
}

#[tokio::test]
async fn fail_harvest_sync_rewards_with_wrong_validator_stake_account() {
    let mut program_test = new_program_test();
    let vote = add_vote_account(
        &mut program_test,
        &Pubkey::new_unique(),
        &Pubkey::new_unique(),
    );
    let mut context = program_test.start_with_context().await;
    let slot = context.genesis_config().epoch_schedule.first_normal_slot + 1;
    context.warp_to_slot(slot).unwrap();

    // Given a config, validator stake and sol staker stake accounts with 1 SOL staked.

    // default sync_rewards_lamports = 1_000_000 (0.001 SOL)
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new_with_vote(&mut context, &config_manager.config, vote).await;
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        1_000_000_000, // 1 SOL staked
    )
    .await;

    // And we another validator stake account.

    let another_validator_stake = ValidatorStakeManager::new(&mut context, &config_manager.config)
        .await
        .stake;

    // When we try to harvest rewards for syncing the SOL stake with the wrong validator stake account.

    let destination = Pubkey::new_unique();
    context.set_account(
        &destination,
        &AccountSharedData::from(Account {
            // amount to cover the account rent
            lamports: 100_000_000,
            ..Default::default()
        }),
    );

    let sync_ix = HarvestSyncRewardsBuilder::new()
        .config(config_manager.config)
        .sol_staker_stake(sol_staker_stake_manager.stake)
        .validator_stake(another_validator_stake) // <- wrong validator stake account
        .sol_stake(sol_staker_stake_manager.sol_stake)
        .destination(destination)
        .sol_stake_view_program(paladin_sol_stake_view_program_client::ID)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[sync_ix],
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
async fn fail_harvest_sync_rewards_with_wrong_config_account() {
    let mut program_test = new_program_test();
    let vote = add_vote_account(
        &mut program_test,
        &Pubkey::new_unique(),
        &Pubkey::new_unique(),
    );
    let mut context = program_test.start_with_context().await;
    let slot = context.genesis_config().epoch_schedule.first_normal_slot + 1;
    context.warp_to_slot(slot).unwrap();

    // Given a config, validator stake and sol staker stake accounts with 1 SOL staked.

    // default sync_rewards_lamports = 1_000_000 (0.001 SOL)
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new_with_vote(&mut context, &config_manager.config, vote).await;
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        1_000_000_000, // 1 SOL staked
    )
    .await;

    // And we create another config account.

    let another_config = ConfigManager::new(&mut context).await.config;

    // When we try to harvest rewards for syncing the SOL stake with the wrong config account.

    let destination = Pubkey::new_unique();
    context.set_account(
        &destination,
        &AccountSharedData::from(Account {
            // amount to cover the account rent
            lamports: 100_000_000,
            ..Default::default()
        }),
    );

    let sync_ix = HarvestSyncRewardsBuilder::new()
        .config(another_config) // <- wrong config account
        .sol_staker_stake(sol_staker_stake_manager.stake)
        .validator_stake(validator_stake_manager.stake)
        .sol_stake(sol_staker_stake_manager.sol_stake)
        .destination(destination)
        .sol_stake_view_program(paladin_sol_stake_view_program_client::ID)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[sync_ix],
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
async fn fail_harvest_sync_rewards_with_invalid_sol_stake_view_program() {
    let mut program_test = new_program_test();
    // add a "fake" sol stake view program
    let fake_sol_stake_view_program = Pubkey::new_unique();
    program_test.add_program(
        "paladin_sol_stake_view_program",
        fake_sol_stake_view_program,
        None,
    );
    let vote = add_vote_account(
        &mut program_test,
        &Pubkey::new_unique(),
        &Pubkey::new_unique(),
    );
    let mut context = program_test.start_with_context().await;
    let slot = context.genesis_config().epoch_schedule.first_normal_slot + 1;
    context.warp_to_slot(slot).unwrap();

    // Given a config, validator stake and sol staker stake accounts with 1 SOL staked.

    // default sync_rewards_lamports = 1_000_000 (0.001 SOL)
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new_with_vote(&mut context, &config_manager.config, vote).await;
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        1_000_000_000, // 1 SOL staked
    )
    .await;

    // When we try to harvest rewards for syncing the SOL stake with an invalid sol stake
    // view program.

    let destination = Pubkey::new_unique();
    context.set_account(
        &destination,
        &AccountSharedData::from(Account {
            // amount to cover the account rent
            lamports: 100_000_000,
            ..Default::default()
        }),
    );

    let sync_ix = HarvestSyncRewardsBuilder::new()
        .config(config_manager.config)
        .sol_staker_stake(sol_staker_stake_manager.stake)
        .validator_stake(validator_stake_manager.stake)
        .sol_stake(sol_staker_stake_manager.sol_stake)
        .destination(destination)
        .sol_stake_view_program(fake_sol_stake_view_program) // <- invalid sol stake view program
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[sync_ix],
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

    assert_instruction_error!(err, InstructionError::IncorrectProgramId);
}
