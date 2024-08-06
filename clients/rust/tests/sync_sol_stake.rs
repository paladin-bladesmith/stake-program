#![cfg(feature = "test-sbf")]

mod setup;

use paladin_stake_program_client::{
    accounts::{SolStakerStake, ValidatorStake},
    instructions::SyncSolStakeBuilder,
};
use setup::{
    config::ConfigManager, setup, sol_staker_stake::SolStakerStakeManager,
    stake::deactivate_stake_account, validator_stake::ValidatorStakeManager,
};
use solana_program_test::tokio;
use solana_sdk::{signature::Signer, transaction::Transaction};

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

    // When we sync the SOL stake.

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
