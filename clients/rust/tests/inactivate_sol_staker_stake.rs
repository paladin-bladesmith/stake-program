#![cfg(feature = "test-sbf")]

mod setup;

use borsh::BorshSerialize;
use paladin_rewards_program_client::accounts::HolderRewards;
use paladin_stake_program_client::{
    accounts::{Config, SolStakerStake, ValidatorStake},
    errors::PaladinStakeProgramError,
    instructions::InactivateSolStakerStakeBuilder,
    pdas::find_sol_staker_stake_pda,
    NullableU64,
};
use setup::{
    config::{create_config, ConfigManager},
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
async fn inactivate_sol_staker_stake_base() {
    let mut context = setup(&[]).await;
    let rent = context.banks_client.get_rent().await.unwrap();
    let schedule = context.genesis_config().epoch_schedule.clone();
    let slot = schedule.first_normal_slot + 1;
    context.warp_to_slot(slot).unwrap();

    // Given a config account (total amount delegated = 100).
    let config_manager = ConfigManager::new(&mut context).await;
    let config = config_manager.config;
    let mut account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config, &account.into());

    // And a validator stake account.
    let validator_stake_manager = ValidatorStakeManager::new(&mut context, &config).await;

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
    stake_account.delegation.active_amount = 100;
    stake_account.delegation.effective_amount = 100;
    stake_account.delegation.deactivating_amount = 50;
    let mut timestamp = context
        .banks_client
        .get_sysvar::<Clock>()
        .await
        .unwrap()
        .unix_timestamp as u64;
    timestamp = timestamp
        .checked_sub(config_account.cooldown_time_seconds)
        .unwrap();
    stake_account.delegation.deactivation_timestamp = NullableU64::from(timestamp);
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &account.into());

    // And we create the holder rewards account for the vault account.
    let (vault_holder_rewards, _) = HolderRewards::find_pda(&config_manager.vault);
    let vault_holder_rewards_state = HolderRewards {
        last_accumulated_rewards_per_token: 0,
        unharvested_rewards: 0,
        padding: 0,
    };
    context.set_account(
        &vault_holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&vault_holder_rewards_state).unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    // When we move the deactivated amount to inactive (50 tokens).
    let inactivate_ix = InactivateSolStakerStakeBuilder::new()
        .config(config)
        .sol_staker_stake(sol_staker_stake_manager.stake)
        .sol_staker_stake_authority(sol_staker_stake_manager.authority.pubkey())
        .vault_holder_rewards(vault_holder_rewards)
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[inactivate_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Assert - The inactivation should be successful.
    let account = get_account!(context, sol_staker_stake_manager.stake);
    let stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake_account.delegation.active_amount, 50);
    assert_eq!(stake_account.delegation.effective_amount, 50);
    assert_eq!(stake_account.delegation.deactivating_amount, 0);
    assert_eq!(stake_account.delegation.inactive_amount, 50);
    assert!(stake_account
        .delegation
        .deactivation_timestamp
        .value()
        .is_none());

    // Assert - The total delegated on the config was updated.
    let account = get_account!(context, config);
    let config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(config_account.token_amount_effective, 50);
}

#[tokio::test]
async fn fail_inactivate_sol_staker_stake_with_no_deactivated_amount() {
    let mut context = setup(&[]).await;
    let rent = context.banks_client.get_rent().await.unwrap();

    // Given a config account (total amount delegated = 100).
    let config_manager = ConfigManager::new(&mut context).await;
    let config = config_manager.config;
    let mut account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config, &account.into());

    // And a validator stake account.
    let validator_stake_manager = ValidatorStakeManager::new(&mut context, &config).await;

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
    stake_account.delegation.active_amount = 100;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &account.into());

    // And we create the holder rewards account for the vault account.
    let (vault_holder_rewards, _) = HolderRewards::find_pda(&config_manager.vault);
    let vault_holder_rewards_state = HolderRewards {
        last_accumulated_rewards_per_token: 0,
        unharvested_rewards: 0,
        padding: 0,
    };
    context.set_account(
        &vault_holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&vault_holder_rewards_state).unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    // When we try to inactivate the stake without any deactivated amount.
    let inactivate_ix = InactivateSolStakerStakeBuilder::new()
        .config(config)
        .sol_staker_stake(sol_staker_stake_manager.stake)
        .sol_staker_stake_authority(sol_staker_stake_manager.authority.pubkey())
        .vault_holder_rewards(vault_holder_rewards)
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
    let mut context = setup(&[]).await;
    let rent = context.banks_client.get_rent().await.unwrap();

    // Given a config account (total amount delegated = 100).
    let config_manager = ConfigManager::new(&mut context).await;
    let config = config_manager.config;
    let mut account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config, &account.into());

    // And a validator stake account.
    let validator_stake_manager = ValidatorStakeManager::new(&mut context, &config).await;

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
    stake_account.delegation.active_amount = 100;
    stake_account.delegation.effective_amount = 100;
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

    // And we create the holder rewards account for the vault account.
    let (vault_holder_rewards, _) = HolderRewards::find_pda(&config_manager.vault);
    let vault_holder_rewards_state = HolderRewards {
        last_accumulated_rewards_per_token: 0,
        unharvested_rewards: 0,
        padding: 0,
    };
    context.set_account(
        &vault_holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&vault_holder_rewards_state).unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    // And we create a second config.
    let wrong_config = create_config(&mut context).await;

    // When we try to inactivate the stake with the wrong config account.
    let inactivate_ix = InactivateSolStakerStakeBuilder::new()
        .config(wrong_config) // <- wrong config
        .sol_staker_stake(sol_staker_stake_manager.stake)
        .sol_staker_stake_authority(sol_staker_stake_manager.authority.pubkey())
        .vault_holder_rewards(vault_holder_rewards)
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
    let mut context = setup(&[]).await;
    let rent = context.banks_client.get_rent().await.unwrap();

    // Given a config and validator stake accounts.
    let config_manager = ConfigManager::new(&mut context).await;
    let config = config_manager.config;

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

    // And we create the holder rewards account for the vault account.
    let (vault_holder_rewards, _) = HolderRewards::find_pda(&config_manager.vault);
    let vault_holder_rewards_state = HolderRewards {
        last_accumulated_rewards_per_token: 0,
        unharvested_rewards: 0,
        padding: 0,
    };
    context.set_account(
        &vault_holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&vault_holder_rewards_state).unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    // When we try to deactivate from an uninitialized stake account.
    let inactivate_ix = InactivateSolStakerStakeBuilder::new()
        .config(config)
        .sol_staker_stake(sol_staker_stake_pda)
        .sol_staker_stake_authority(Pubkey::new_unique())
        .vault_holder_rewards(vault_holder_rewards)
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
    let mut context = setup(&[]).await;
    let rent = context.banks_client.get_rent().await.unwrap();

    // Given a config account (total amount delegated = 100).
    let config_manager = ConfigManager::with_args(
        &mut context,
        10,        /* cooldown 10 seconds */
        500,       /* basis points 5%     */
        1_000_000, /* sync rewards lamports 0.001 SOL */
    )
    .await;
    let config = config_manager.config;
    let mut account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config, &account.into());

    // And a validator stake account (total amount staked = 100).
    let validator_stake_manager = ValidatorStakeManager::new(&mut context, &config).await;

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
    stake_account.delegation.active_amount = 100;
    stake_account.delegation.effective_amount = 100;
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

    // And we create the holder rewards account for the vault account.
    let (vault_holder_rewards, _) = HolderRewards::find_pda(&config_manager.vault);
    let vault_holder_rewards_state = HolderRewards {
        last_accumulated_rewards_per_token: 0,
        unharvested_rewards: 0,
        padding: 0,
    };
    context.set_account(
        &vault_holder_rewards,
        &Account {
            lamports: rent.minimum_balance(HolderRewards::LEN),
            data: borsh::to_vec(&vault_holder_rewards_state).unwrap(),
            owner: paladin_rewards_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    // When we try to move the deactivated amount to inactive before the end of
    // the cooldown period.
    let inactivate_ix = InactivateSolStakerStakeBuilder::new()
        .config(config)
        .sol_staker_stake(sol_staker_stake_manager.stake)
        .sol_staker_stake_authority(sol_staker_stake_manager.authority.pubkey())
        .vault_holder_rewards(vault_holder_rewards)
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
