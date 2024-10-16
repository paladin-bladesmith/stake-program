#![cfg(feature = "test-sbf")]

mod setup;

use borsh::BorshSerialize;
use paladin_stake_program_client::{
    accounts::{Config, SolStakerStake, ValidatorStake},
    errors::PaladinStakeProgramError,
    instructions::DeactivateStakeBuilder,
    pdas::find_validator_stake_pda,
};
use setup::{
    config::{create_config, create_config_with_args, ConfigManager},
    setup,
    sol_staker_stake::SolStakerStakeManager,
    validator_stake::{create_validator_stake, ValidatorStakeManager},
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
async fn validator_stake_deactivate_stake() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config account and a validator's vote account.

    let config_manager = ConfigManager::new(&mut context).await;
    // "manually" set the total amount delegated
    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    // And a stake account.

    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    // "manually" set the amount to 100
    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    stake_account.delegation.active_amount = 100;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    // When we deactivate an amount from the stake account.

    let deactivate_ix = DeactivateStakeBuilder::new()
        .config(config_manager.config)
        .stake(validator_stake_manager.stake)
        .stake_authority(validator_stake_manager.authority.pubkey())
        .amount(5)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[deactivate_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &validator_stake_manager.authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the deactivation should be successful.

    let account = get_account!(context, validator_stake_manager.stake);
    let stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();

    assert_eq!(stake_account.delegation.active_amount, 100);
    assert_eq!(stake_account.delegation.deactivating_amount, 5);
    assert!(stake_account
        .delegation
        .deactivation_timestamp
        .value()
        .is_some())
}

#[tokio::test]
async fn validator_stake_deactivate_stake_with_active_deactivation() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config account and a validator's vote account.

    let config_manager = ConfigManager::new(&mut context).await;
    // "manually" set the total amount delegated
    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    // And a stake account.

    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    // "manually" set the amount to 100
    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    stake_account.delegation.active_amount = 100;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    // And we deactivate an amount (5) from the stake account.

    let deactivate_ix = DeactivateStakeBuilder::new()
        .config(config_manager.config)
        .stake(validator_stake_manager.stake)
        .stake_authority(validator_stake_manager.authority.pubkey())
        .amount(5)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[deactivate_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &validator_stake_manager.authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    let account = get_account!(context, validator_stake_manager.stake);
    let stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake_account.delegation.deactivating_amount, 5);

    let mut clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();
    // updated timestamp
    let updated_timestamp = clock.unix_timestamp.saturating_add(1000);
    clock.unix_timestamp = updated_timestamp;
    context.set_sysvar::<Clock>(&clock);

    // When we deactivate a different amount from the stake account
    // with an active deactivation.

    let deactivate_ix = DeactivateStakeBuilder::new()
        .config(config_manager.config)
        .stake(validator_stake_manager.stake)
        .stake_authority(validator_stake_manager.authority.pubkey())
        .amount(1)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[deactivate_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &validator_stake_manager.authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the deactivation should have the updated amount and timestamp.

    let account = get_account!(context, validator_stake_manager.stake);
    let stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake_account.delegation.deactivating_amount, 1);
    assert_eq!(
        updated_timestamp as u64,
        stake_account
            .delegation
            .deactivation_timestamp
            .value()
            .unwrap()
    );
}

#[tokio::test]
async fn fail_validator_stake_deactivate_stake_with_amount_greater_than_stake_amount() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config account and a validator's vote account.

    let config_manager = ConfigManager::new(&mut context).await;
    // "manually" set the total amount delegated
    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    // And a stake account.

    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    // "manually" set the amount to 100
    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    stake_account.delegation.active_amount = 100;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    // When we try to deactivate an amount greater than the staked amount.

    let deactivate_ix = DeactivateStakeBuilder::new()
        .config(config_manager.config)
        .stake(validator_stake_manager.stake)
        .stake_authority(validator_stake_manager.authority.pubkey())
        .amount(150)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[deactivate_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &validator_stake_manager.authority],
        context.last_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Then we expect an error.

    assert_custom_error!(err, PaladinStakeProgramError::InsufficientStakeAmount);
}

#[tokio::test]
async fn fail_validator_stake_deactivate_stake_with_invalid_authority() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config account and a validator's vote account.

    let config_manager = ConfigManager::new(&mut context).await;
    // "manually" set the total amount delegated
    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    // And a stake account.

    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    // "manually" set the amount to 100
    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    stake_account.delegation.active_amount = 100;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    // When we try to deactivate with an invalid authority.

    let fake_authority = Keypair::new();

    let deactivate_ix = DeactivateStakeBuilder::new()
        .config(config_manager.config)
        .stake(validator_stake_manager.stake)
        .stake_authority(fake_authority.pubkey())
        .amount(50)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[deactivate_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &fake_authority],
        context.last_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Then we expect an error.

    assert_custom_error!(err, PaladinStakeProgramError::InvalidAuthority);
}

#[tokio::test]
async fn validator_stake_deactivate_stake_with_zero_amount() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config account and a validator's vote account.

    let config_manager = ConfigManager::new(&mut context).await;
    // "manually" set the total amount delegated
    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    // And a stake account.

    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    // "manually" set the amount to 100
    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    stake_account.delegation.active_amount = 100;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    // When we deactivate with zero amount.

    let deactivate_ix = DeactivateStakeBuilder::new()
        .config(config_manager.config)
        .stake(validator_stake_manager.stake)
        .stake_authority(validator_stake_manager.authority.pubkey())
        .amount(0)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[deactivate_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &validator_stake_manager.authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the deactivation should be cancelled.

    let account = get_account!(context, validator_stake_manager.stake);
    let stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();

    assert_eq!(stake_account.delegation.deactivating_amount, 0);
    assert!(stake_account
        .delegation
        .deactivation_timestamp
        .value()
        .is_none())
}

#[tokio::test]
async fn fail_deactivate_stake_with_uninitialized_stake_account() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config account and a validator's vote account.

    let config = create_config_with_args(
        &mut context,
        1,           /* cooldown 1 second */
        1000,        /* basis points 10%  */
        100_000_000, /* sync_rewards_lamports 0.001 SOL */
    )
    .await;

    // And an uninitialized stake account.

    let validator = Pubkey::new_unique();
    let authority = Keypair::new();
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

    // When we try to deactivate from an uninitialized stake account.

    let deactivate_ix = DeactivateStakeBuilder::new()
        .config(config)
        .stake(stake_pda)
        .stake_authority(authority.pubkey())
        .amount(0)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[deactivate_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &authority],
        context.last_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Then we expect an error.

    assert_instruction_error!(err, InstructionError::InvalidAccountData);
}

#[tokio::test]
async fn fail_validator_stake_deactivate_stake_with_maximum_deactivation_amount_exceeded() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config account and a validator's vote account.

    let config_manager = ConfigManager::new(&mut context).await;
    // "manually" set the total amount delegated
    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    // And a stake account.

    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    // "manually" set the amount to 100
    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    stake_account.delegation.active_amount = 100;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    // When we try to deactivate a greater amount than the maximum allowed.

    let deactivate_ix = DeactivateStakeBuilder::new()
        .config(config_manager.config)
        .stake(validator_stake_manager.stake)
        .stake_authority(validator_stake_manager.authority.pubkey())
        .amount(100) // <- equivalent to 100% of the stake
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[deactivate_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &validator_stake_manager.authority],
        context.last_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Then we expect an error.

    assert_custom_error!(
        err,
        PaladinStakeProgramError::MaximumDeactivationAmountExceeded
    );
}

#[tokio::test]
async fn fail_deactivate_stake_with_uninitialized_config_account() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config account and a validator's vote account.

    let config = create_config(&mut context).await;
    // "manually" set the total amount delegated
    let account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 100;

    let updated_config = Account {
        lamports: account.lamports,
        data: config_account.try_to_vec().unwrap(),
        owner: account.owner,
        executable: account.executable,
        rent_epoch: account.rent_epoch,
    };
    context.set_account(&config, &updated_config.into());

    let validator = Pubkey::new_unique();
    let authority = Keypair::new();
    let vote = create_vote_account(&mut context, &validator, &authority.pubkey()).await;

    // And a stake account.

    let stake_pda = create_validator_stake(&mut context, &vote, &config).await;

    let account = get_account!(context, stake_pda);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the amount to 100
    stake_account.delegation.active_amount = 100;

    let updated_stake = Account {
        lamports: account.lamports,
        data: stake_account.try_to_vec().unwrap(),
        owner: account.owner,
        executable: account.executable,
        rent_epoch: account.rent_epoch,
    };

    context.set_account(&stake_pda, &updated_stake.into());

    // And we uninitialize the config account.

    context.set_account(
        &config,
        &AccountSharedData::from(Account {
            lamports: 100_000_000,
            data: vec![5; Config::LEN],
            owner: paladin_stake_program_client::ID,
            ..Default::default()
        }),
    );

    // When we try to deactivate an amount from the stake account.

    let deactivate_ix = DeactivateStakeBuilder::new()
        .config(config)
        .stake(stake_pda)
        .stake_authority(authority.pubkey())
        .amount(5)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[deactivate_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &authority],
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
async fn sol_staker_stake_deactivate_stake() {
    let mut context = setup().await;

    // Given a config account and a validator's vote account.

    let config_manager = ConfigManager::new(&mut context).await;
    // "manually" set the total amount delegated
    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    // And a validator stake account.
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    // And a SOL staker stake account.
    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        1_000_000_000,
    )
    .await;
    // "manually" set the amount to 100
    let mut account = get_account!(context, sol_staker_stake_manager.stake);
    let mut stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    stake_account.delegation.active_amount = 100;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &account.into());

    // When we deactivate an amount from the SOL staker stake account.

    let deactivate_ix = DeactivateStakeBuilder::new()
        .config(config_manager.config)
        .stake(sol_staker_stake_manager.stake)
        .stake_authority(sol_staker_stake_manager.authority.pubkey())
        .amount(5)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[deactivate_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &sol_staker_stake_manager.authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the deactivation should be successful.

    let account = get_account!(context, sol_staker_stake_manager.stake);
    let stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();

    assert_eq!(stake_account.delegation.active_amount, 100);
    assert_eq!(stake_account.delegation.deactivating_amount, 5);
    assert!(stake_account
        .delegation
        .deactivation_timestamp
        .value()
        .is_some())
}

#[tokio::test]
async fn sol_staker_stake_deactivate_stake_with_active_deactivation() {
    let mut context = setup().await;

    // Given a config account and a validator's vote account.

    let config_manager = ConfigManager::new(&mut context).await;
    // "manually" set the total amount delegated
    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    // And a stake account.

    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    // "manually" set the amount to 100
    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    stake_account.delegation.active_amount = 100;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    // And a SOL staker stake account.

    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        1_000_000_000,
    )
    .await;
    // "manually" set the amount to 100
    let mut account = get_account!(context, sol_staker_stake_manager.stake);
    let mut stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    stake_account.delegation.active_amount = 100;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &account.into());

    // And we deactivate an amount (5) from the stake account.

    let deactivate_ix = DeactivateStakeBuilder::new()
        .config(config_manager.config)
        .stake(sol_staker_stake_manager.stake)
        .stake_authority(sol_staker_stake_manager.authority.pubkey())
        .amount(5)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[deactivate_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &sol_staker_stake_manager.authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    let account = get_account!(context, sol_staker_stake_manager.stake);
    let stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake_account.delegation.deactivating_amount, 5);

    let mut clock = context.banks_client.get_sysvar::<Clock>().await.unwrap();
    // updated timestamp
    let updated_timestamp = clock.unix_timestamp.saturating_add(1000);
    clock.unix_timestamp = updated_timestamp;
    context.set_sysvar::<Clock>(&clock);

    // When we deactivate a different amount from the stake account
    // with an active deactivation.

    let deactivate_ix = DeactivateStakeBuilder::new()
        .config(config_manager.config)
        .stake(sol_staker_stake_manager.stake)
        .stake_authority(sol_staker_stake_manager.authority.pubkey())
        .amount(1)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[deactivate_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &sol_staker_stake_manager.authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the deactivation should have the updated amount and timestamp.

    let account = get_account!(context, sol_staker_stake_manager.stake);
    let stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake_account.delegation.deactivating_amount, 1);
    assert_eq!(
        updated_timestamp as u64,
        stake_account
            .delegation
            .deactivation_timestamp
            .value()
            .unwrap()
    );
}

#[tokio::test]
async fn fail_sol_staker_stake_deactivate_stake_with_amount_greater_than_stake_amount() {
    let mut context = setup().await;

    // Given a config account and a validator's vote account.

    let config_manager = ConfigManager::new(&mut context).await;
    // "manually" set the total amount delegated
    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    // And a stake account.

    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    // "manually" set the amount to 100
    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    stake_account.delegation.active_amount = 100;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    // And a SOL staker stake account.

    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        1_000_000_000,
    )
    .await;
    // "manually" set the amount to 100
    let mut account = get_account!(context, sol_staker_stake_manager.stake);
    let mut stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    stake_account.delegation.active_amount = 100;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &account.into());

    // When we try to deactivate an amount greater than the staked amount.

    let deactivate_ix = DeactivateStakeBuilder::new()
        .config(config_manager.config)
        .stake(sol_staker_stake_manager.stake)
        .stake_authority(sol_staker_stake_manager.authority.pubkey())
        .amount(150)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[deactivate_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &sol_staker_stake_manager.authority],
        context.last_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Then we expect an error.

    assert_custom_error!(err, PaladinStakeProgramError::InsufficientStakeAmount);
}

#[tokio::test]
async fn fail_sol_staker_stake_deactivate_stake_with_maximum_deactivation_amount_exceeded() {
    let mut context = setup().await;

    // Given a config account and a validator's vote account.

    let config_manager = ConfigManager::new(&mut context).await;
    // "manually" set the total amount delegated
    let mut account = get_account!(context, config_manager.config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_effective = 100;
    account.data = config_account.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &account.into());

    // And a stake account.

    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    // "manually" set the amount to 100
    let mut account = get_account!(context, validator_stake_manager.stake);
    let mut stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    stake_account.delegation.active_amount = 100;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&validator_stake_manager.stake, &account.into());

    // And a SOL staker stake account.

    let sol_staker_stake_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        1_000_000_000,
    )
    .await;
    // "manually" set the amount to 100
    let mut account = get_account!(context, sol_staker_stake_manager.stake);
    let mut stake_account = SolStakerStake::from_bytes(account.data.as_ref()).unwrap();
    stake_account.delegation.active_amount = 100;
    account.data = stake_account.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &account.into());

    // When we try to deactivate a greater amount than the maximum allowed.

    let deactivate_ix = DeactivateStakeBuilder::new()
        .config(config_manager.config)
        .stake(sol_staker_stake_manager.stake)
        .stake_authority(sol_staker_stake_manager.authority.pubkey())
        .amount(100) // <- equivalent to 100% of the stake
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[deactivate_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &sol_staker_stake_manager.authority],
        context.last_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Then we expect an error.

    assert_custom_error!(
        err,
        PaladinStakeProgramError::MaximumDeactivationAmountExceeded
    );
}
