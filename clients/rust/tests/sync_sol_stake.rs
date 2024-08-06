#![cfg(feature = "test-sbf")]

mod setup;

use paladin_stake_program_client::{
    accounts::{SolStakerStake, ValidatorStake},
    instructions::SyncSolStakeBuilder,
};
use setup::{
    config::ConfigManager, new_program_test, setup, sol_staker_stake::SolStakerStakeManager,
    stake::deactivate_stake_account, validator_stake::ValidatorStakeManager,
    vote::add_vote_account,
};
use solana_program_test::tokio;
use solana_sdk::{pubkey::Pubkey, signature::Signer, transaction::Transaction};

#[tokio::test]
async fn sync_sol_stake_when_deactivating() {
    let mut context = setup().await;

    // Given a config, validator stake and sol staker stake accounts with 5 SOL staked.

    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let sol_staker_staker_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        5_000_000_000, // 5 SOL staked
    )
    .await;

    // And the SOL staker stake and validator stake accounts are correctly synced.

    let account = get_account!(context, sol_staker_staker_manager.stake);
    let stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();

    assert_eq!(stake_account.lamports_amount, 5_000_000_000);

    let account = get_account!(context, validator_stake_manager.stake);
    let validator_stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        validator_stake_account.total_staked_lamports_amount,
        5_000_000_000
    );

    // And we deactivate the stake.

    deactivate_stake_account(
        &mut context,
        &stake_account.sol_stake,
        &sol_staker_staker_manager.authority,
    )
    .await;

    // When we sync the SOL stake after deactivating the SOL stake.

    let sync_ix = SyncSolStakeBuilder::new()
        .config(config_manager.config)
        .sol_staker_stake(sol_staker_staker_manager.stake)
        .validator_stake(validator_stake_manager.stake)
        .sol_stake(sol_staker_staker_manager.sol_stake)
        .sol_stake_view_program(paladin_sol_stake_view_program_client::ID)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[sync_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the SOL amounts are correctly synced (0 SOL staked).

    let account = get_account!(context, sol_staker_staker_manager.stake);
    let stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();

    assert_eq!(stake_account.lamports_amount, 0);

    let account = get_account!(context, validator_stake_manager.stake);
    let validator_stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(validator_stake_account.total_staked_lamports_amount, 0);
}

#[tokio::test]
async fn sync_sol_stake_when_inactive() {
    let mut program_test = new_program_test();
    let vote = add_vote_account(
        &mut program_test,
        &Pubkey::new_unique(),
        &Pubkey::new_unique(),
    );
    let mut context = program_test.start_with_context().await;
    let slot = context.genesis_config().epoch_schedule.first_normal_slot + 1;
    context.warp_to_slot(slot).unwrap();

    // Given a config, validator stake and sol staker stake accounts with 5 SOL staked.

    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new_with_vote(&mut context, &config_manager.config, vote).await;
    let sol_staker_staker_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        5_000_000_000, // 5 SOL staked
    )
    .await;

    // And the SOL staker stake and validator stake accounts are correctly synced.

    let account = get_account!(context, sol_staker_staker_manager.stake);
    let stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();

    assert_eq!(stake_account.lamports_amount, 5_000_000_000);

    let account = get_account!(context, validator_stake_manager.stake);
    let validator_stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        validator_stake_account.total_staked_lamports_amount,
        5_000_000_000
    );

    // And we deactivate the stake and wait for the deactivation to take effect.

    let slot = slot + context.genesis_config().epoch_schedule.slots_per_epoch;
    context.warp_to_slot(slot).unwrap();

    deactivate_stake_account(
        &mut context,
        &stake_account.sol_stake,
        &sol_staker_staker_manager.authority,
    )
    .await;

    let slot = slot + context.genesis_config().epoch_schedule.slots_per_epoch;
    context.warp_to_slot(slot).unwrap();

    // When we sync the SOL stake after the stake has been inactive.

    let sync_ix = SyncSolStakeBuilder::new()
        .config(config_manager.config)
        .sol_staker_stake(sol_staker_staker_manager.stake)
        .validator_stake(validator_stake_manager.stake)
        .sol_stake(sol_staker_staker_manager.sol_stake)
        .sol_stake_view_program(paladin_sol_stake_view_program_client::ID)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[sync_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the SOL amounts are correctly synced (0 SOL staked).

    let account = get_account!(context, sol_staker_staker_manager.stake);
    let stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();

    assert_eq!(stake_account.lamports_amount, 0);

    let account = get_account!(context, validator_stake_manager.stake);
    let validator_stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(validator_stake_account.total_staked_lamports_amount, 0);
}
