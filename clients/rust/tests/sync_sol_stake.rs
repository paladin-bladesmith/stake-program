#![cfg(feature = "test-sbf")]

mod setup;

use paladin_stake_program_client::{
    accounts::{Config, SolStakerStake, ValidatorStake},
    errors::PaladinStakeProgramError,
    instructions::SyncSolStakeBuilder,
};
use setup::{
    config::ConfigManager,
    new_program_test, setup,
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
    signature::{Keypair, Signer},
    stake::state::{Authorized, Lockup},
    transaction::Transaction,
};

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

#[tokio::test]
async fn sync_sol_stake_when_effective() {
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

    // And we wait until the stake is effective.

    let slot = slot + context.genesis_config().epoch_schedule.slots_per_epoch;
    context.warp_to_slot(slot).unwrap();

    // When we sync the SOL stake after the stake is effective.

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

    // Then the SOL amounts are correctly synced (5 SOL staked).

    let account = get_account!(context, sol_staker_staker_manager.stake);
    let stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();

    assert_eq!(stake_account.lamports_amount, 5_000_000_000);

    let account = get_account!(context, validator_stake_manager.stake);
    let validator_stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        validator_stake_account.total_staked_lamports_amount,
        5_000_000_000
    );
}

#[tokio::test]
async fn sync_sol_stake_when_activating() {
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

    // When we sync the SOL stake while the SOL stake is activating.

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

    // Then the SOL amounts are correctly synced (5 SOL staked).

    let account = get_account!(context, sol_staker_staker_manager.stake);
    let stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();

    assert_eq!(stake_account.lamports_amount, 5_000_000_000);

    let account = get_account!(context, validator_stake_manager.stake);
    let validator_stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        validator_stake_account.total_staked_lamports_amount,
        5_000_000_000
    );
}

#[tokio::test]
async fn fail_sync_sol_stake_with_wrong_config_account() {
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

    // And we create another config account.

    let another_config = ConfigManager::new(&mut context).await;

    // When we try to sync the SOL stake with the wrong config account.

    let sync_ix = SyncSolStakeBuilder::new()
        .config(another_config.config)
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
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Then we expect an error.

    assert_instruction_error!(err, InstructionError::InvalidSeeds);
}

#[tokio::test]
async fn fail_sync_sol_stake_with_wrong_sol_stake_account() {
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

    // And we create another SOL stake account.

    let another_sol_stake = Keypair::new();
    let authority = Keypair::new();

    create_stake_account(
        &mut context,
        &another_sol_stake,
        &Authorized::auto(&authority.pubkey()),
        &Lockup::default(),
        0,
    )
    .await;

    // When we try to sync with the wrong SOL stake.

    let sync_ix = SyncSolStakeBuilder::new()
        .config(config_manager.config)
        .sol_staker_stake(sol_staker_staker_manager.stake)
        .validator_stake(validator_stake_manager.stake)
        .sol_stake(another_sol_stake.pubkey())
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
async fn fail_sync_sol_stake_with_wrong_validator_stake() {
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

    // And we another validator stake account.

    let another_validator_stake = ValidatorStakeManager::new(&mut context, &config_manager.config)
        .await
        .stake;

    // When we try to sync with the wrong validator stake account.

    let sync_ix = SyncSolStakeBuilder::new()
        .config(config_manager.config)
        .sol_staker_stake(sol_staker_staker_manager.stake)
        .validator_stake(another_validator_stake)
        .sol_stake(sol_staker_staker_manager.sol_stake)
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
async fn fail_sync_sol_stake_with_uninitialized_config() {
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

    // And we uninitialize the config account.

    context.set_account(
        &config_manager.config,
        &AccountSharedData::from(Account {
            lamports: 100_000_000,
            data: vec![5; Config::LEN],
            owner: paladin_stake_program_client::ID,
            ..Default::default()
        }),
    );

    // When we try to sync with a unitialized config account.

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
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Then we expect an error.

    assert_instruction_error!(err, InstructionError::UninitializedAccount);
}

#[tokio::test]
async fn fail_sync_sol_stake_with_uninitialized_validator_stake() {
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

    // And we uninitialize the validator stake account.

    context.set_account(
        &validator_stake_manager.stake,
        &AccountSharedData::from(Account {
            lamports: 100_000_000,
            data: vec![5; ValidatorStake::LEN],
            owner: paladin_stake_program_client::ID,
            ..Default::default()
        }),
    );

    // When we try to sync with a unitialized validator stake account.

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
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Then we expect an error.

    assert_instruction_error!(err, InstructionError::UninitializedAccount);
}

#[tokio::test]
async fn fail_sync_sol_stake_with_invalid_sol_stake_view_program() {
    let mut program_test = new_program_test();
    // add a "fake" sol stake view program
    let fake_sol_stake_view_program = Pubkey::new_unique();
    program_test.add_program(
        "paladin_sol_stake_view_program",
        fake_sol_stake_view_program,
        None,
    );
    let mut context = program_test.start_with_context().await;

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

    // When we try to sync with a unitialized validator stake account.

    let sync_ix = SyncSolStakeBuilder::new()
        .config(config_manager.config)
        .sol_staker_stake(sol_staker_staker_manager.stake)
        .validator_stake(validator_stake_manager.stake)
        .sol_stake(sol_staker_staker_manager.sol_stake)
        .sol_stake_view_program(fake_sol_stake_view_program) // <- fake sol stake view program
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
