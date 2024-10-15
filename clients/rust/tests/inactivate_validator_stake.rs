#![cfg(feature = "test-sbf")]

mod setup;

use borsh::BorshSerialize;
use paladin_rewards_program_client::accounts::HolderRewards;
use paladin_stake_program_client::{
    accounts::{Config, ValidatorStake},
    errors::PaladinStakeProgramError,
    instructions::InactivateValidatorStakeBuilder,
    pdas::find_validator_stake_pda,
    NullableU64,
};
use setup::{
    config::{create_config, ConfigManager},
    validator_stake::create_validator_stake,
    vote::create_vote_account,
};
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{
    account::{Account, AccountSharedData},
    clock::Clock,
    instruction::InstructionError,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

#[tokio::test]
async fn inactivate_validator_stake() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;
    let rent = context.banks_client.get_rent().await.unwrap();

    // Given a config account (total amount delegated = 100).
    let config_manager = ConfigManager::new(&mut context).await;
    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    // And a validator stake account (amount = 100).
    let validator = Pubkey::new_unique();
    let authority = Keypair::new();
    let vote = create_vote_account(&mut context, &validator, &authority.pubkey()).await;
    let stake_pda = create_validator_stake(&mut context, &vote, &config_manager.config).await;

    let mut account = get_account!(context, stake_pda);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    stake_account.delegation.amount = 100;
    stake_account.delegation.effective_amount = 100;
    stake_account.delegation.deactivating_amount = 50;
    stake_account.total_staked_lamports_amount = 100;
    let mut timestamp = context
        .banks_client
        .get_sysvar::<Clock>()
        .await
        .unwrap()
        .unix_timestamp as u64;
    timestamp = timestamp.saturating_sub(config_account.cooldown_time_seconds);
    stake_account.delegation.deactivation_timestamp = NullableU64::from(timestamp);
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&stake_pda, &account.into());

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
    let inactivate_ix = InactivateValidatorStakeBuilder::new()
        .config(config_manager.config)
        .validator_stake(stake_pda)
        .validator_stake_authority(authority.pubkey())
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
    let account = get_account!(context, stake_pda);
    let stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake_account.delegation.amount, 50);
    assert_eq!(stake_account.delegation.effective_amount, 50);
    assert_eq!(stake_account.delegation.deactivating_amount, 0);
    assert_eq!(stake_account.delegation.inactive_amount, 50);
    assert!(stake_account
        .delegation
        .deactivation_timestamp
        .value()
        .is_none());

    // Assert - The total delegated on the config was updated.
    let account = get_account!(context, config_manager.config);
    let config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(config_account.token_amount_effective, 50);
}

#[tokio::test]
async fn fail_inactivate_validator_stake_with_no_deactivated_amount() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;
    let rent = context.banks_client.get_rent().await.unwrap();

    // Given a config account (total amount delegated = 100).
    let config_manager = ConfigManager::new(&mut context).await;
    let config = config_manager.config;
    let mut account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config, &account.into());

    // And a validator stake account (amount = 100).
    let validator = Pubkey::new_unique();
    let authority = Keypair::new();
    let vote = create_vote_account(&mut context, &validator, &authority.pubkey()).await;
    let stake_pda = create_validator_stake(&mut context, &vote, &config).await;
    let mut account = get_account!(context, stake_pda);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    stake_account.delegation.amount = 100;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&stake_pda, &account.into());

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
    let inactivate_ix = InactivateValidatorStakeBuilder::new()
        .config(config)
        .validator_stake(stake_pda)
        .validator_stake_authority(authority.pubkey())
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
async fn fail_inactivate_validator_stake_with_wrong_config() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;
    let rent = context.banks_client.get_rent().await.unwrap();

    // Given a config account (total amount delegated = 100).
    let config_manager = ConfigManager::new(&mut context).await;
    let config = config_manager.config;
    let mut account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config, &account.into());

    // And a validator stake account (amount = 100).
    let validator = Pubkey::new_unique();
    let authority = Keypair::new();
    let vote = create_vote_account(&mut context, &validator, &authority.pubkey()).await;
    let stake_pda = create_validator_stake(&mut context, &vote, &config).await;
    let mut account = get_account!(context, stake_pda);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    stake_account.delegation.amount = 100;
    stake_account.delegation.effective_amount = 100;
    stake_account.delegation.deactivating_amount = 50;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&stake_pda, &account.into());

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
    let inactivate_ix = InactivateValidatorStakeBuilder::new()
        .config(wrong_config) // <- wrong config
        .validator_stake(stake_pda)
        .validator_stake_authority(authority.pubkey())
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
async fn fail_inactivate_validator_stake_with_uninitialized_stake_account() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;
    let rent = context.banks_client.get_rent().await.unwrap();

    // Given a config account and a validator's vote account.
    let config_manager = ConfigManager::new(&mut context).await;
    let config = config_manager.config;
    let validator = Pubkey::new_unique();

    // And an uninitialized validator stake account.
    let (stake_pda, _) = find_validator_stake_pda(&validator, &config);
    context.set_account(
        &stake_pda,
        &AccountSharedData::from(Account {
            lamports: 100_000_000,
            data: vec![5; ValidatorStake::LEN],
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
    let inactivate_ix = InactivateValidatorStakeBuilder::new()
        .config(config)
        .validator_stake(stake_pda)
        .validator_stake_authority(Pubkey::new_unique())
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
async fn fail_inactivate_validator_stake_with_active_cooldown() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;
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

    // And a validator stake account (amount = 100) with 50 tokens deactivated.
    let validator = Pubkey::new_unique();
    let authority = Keypair::new();
    let vote = create_vote_account(&mut context, &validator, &authority.pubkey()).await;
    let stake_pda = create_validator_stake(&mut context, &vote, &config).await;
    let mut account = get_account!(context, stake_pda);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the stake values
    stake_account.delegation.amount = 100;
    stake_account.delegation.deactivating_amount = 50;

    let timestamp = context
        .banks_client
        .get_sysvar::<Clock>()
        .await
        .unwrap()
        .unix_timestamp;
    stake_account.delegation.deactivation_timestamp = NullableU64::from(timestamp as u64);
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&stake_pda, &account.into());

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
    let inactivate_ix = InactivateValidatorStakeBuilder::new()
        .config(config)
        .validator_stake(stake_pda)
        .validator_stake_authority(authority.pubkey())
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
