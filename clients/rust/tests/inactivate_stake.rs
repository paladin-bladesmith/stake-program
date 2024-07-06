#![cfg(feature = "test-sbf")]

mod setup;

use borsh::BorshSerialize;
use paladin_stake::{
    accounts::{Config, Stake},
    instructions::InactivateStakeBuilder,
    pdas::find_stake_pda,
    NullableU64,
};
use setup::{config::create_config, stake::create_stake, vote::create_vote_account};
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
async fn inactivate_stake() {
    let mut context = ProgramTest::new("stake_program", paladin_stake::ID, None)
        .start_with_context()
        .await;

    // Given a config account (total amount delegated = 100).

    let config = create_config(&mut context).await;
    // "manually" set the total amount delegated
    let account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_delegated = 100;

    let updated_config = Account {
        lamports: account.lamports,
        data: config_account.try_to_vec().unwrap(),
        owner: account.owner,
        executable: account.executable,
        rent_epoch: account.rent_epoch,
    };
    context.set_account(&config, &updated_config.into());

    // And a stake account (amount = 100).

    let validator = Pubkey::new_unique();
    let authority = Keypair::new();
    let vote = create_vote_account(&mut context, &validator, &authority.pubkey()).await;

    let stake_pda = create_stake(&mut context, &validator, &vote, &config).await;

    let account = get_account!(context, stake_pda);
    let mut stake_account = Stake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the stake values
    stake_account.amount = 100;
    stake_account.deactivating_amount = 50;

    let mut timestamp = context
        .banks_client
        .get_sysvar::<Clock>()
        .await
        .unwrap()
        .unix_timestamp;
    timestamp = timestamp.saturating_sub(config_account.cooldown_time_seconds);
    stake_account.deactivation_timestamp = NullableU64::from(timestamp as u64);

    let updated_stake = Account {
        lamports: account.lamports,
        data: stake_account.try_to_vec().unwrap(),
        owner: account.owner,
        executable: account.executable,
        rent_epoch: account.rent_epoch,
    };
    context.set_account(&stake_pda, &updated_stake.into());

    // When we move the deactivated amount to inactive (50 tokens).

    let inactivate_ix = InactivateStakeBuilder::new()
        .config(config)
        .stake(stake_pda)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[inactivate_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the inactivation should be successful.

    let account = get_account!(context, stake_pda);
    let stake_account = Stake::from_bytes(account.data.as_ref()).unwrap();

    assert_eq!(stake_account.amount, 50);
    assert_eq!(stake_account.deactivating_amount, 0);
    assert_eq!(stake_account.inactive_amount, 50);
    assert!(stake_account.deactivation_timestamp.value().is_none());

    // And the total delegated on the config must decrease.

    let account = get_account!(context, config);
    let config_account = Config::from_bytes(account.data.as_ref()).unwrap();

    assert_eq!(config_account.token_amount_delegated, 50);
}

#[tokio::test]
async fn inactivate_stake_with_no_deactivated_amount() {
    let mut context = ProgramTest::new("stake_program", paladin_stake::ID, None)
        .start_with_context()
        .await;

    // Given a config account (total amount delegated = 100).

    let config = create_config(&mut context).await;
    // "manually" set the total amount delegated
    let account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_delegated = 100;

    let updated_config = Account {
        lamports: account.lamports,
        data: config_account.try_to_vec().unwrap(),
        owner: account.owner,
        executable: account.executable,
        rent_epoch: account.rent_epoch,
    };
    context.set_account(&config, &updated_config.into());

    // And a stake account (amount = 100).

    let validator = Pubkey::new_unique();
    let authority = Keypair::new();
    let vote = create_vote_account(&mut context, &validator, &authority.pubkey()).await;

    let stake_pda = create_stake(&mut context, &validator, &vote, &config).await;

    let account = get_account!(context, stake_pda);
    let mut stake_account = Stake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the stake values
    stake_account.amount = 100;

    let updated_stake = Account {
        lamports: account.lamports,
        data: stake_account.try_to_vec().unwrap(),
        owner: account.owner,
        executable: account.executable,
        rent_epoch: account.rent_epoch,
    };
    context.set_account(&stake_pda, &updated_stake.into());

    // When we try to inactivate the stake without any deactivated amount.

    let inactivate_ix = InactivateStakeBuilder::new()
        .config(config)
        .stake(stake_pda)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[inactivate_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the tranasction succeeds and the stake account should
    // remain the same.

    let account = get_account!(context, stake_pda);
    let stake_account = Stake::from_bytes(account.data.as_ref()).unwrap();

    assert_eq!(stake_account.amount, 100);
    assert_eq!(stake_account.deactivating_amount, 0);
    assert_eq!(stake_account.inactive_amount, 0);
    assert!(stake_account.deactivation_timestamp.value().is_none());

    // And the total delegated on the config is the same.

    let account = get_account!(context, config);
    let config_account = Config::from_bytes(account.data.as_ref()).unwrap();

    assert_eq!(config_account.token_amount_delegated, 100);
}

#[tokio::test]
async fn fail_inactivate_stake_with_wrong_config() {
    let mut context = ProgramTest::new("stake_program", paladin_stake::ID, None)
        .start_with_context()
        .await;

    // Given a config account (total amount delegated = 100).

    let config = create_config(&mut context).await;
    // "manually" set the total amount delegated
    let account = get_account!(context, config);
    let mut config_account = Config::from_bytes(account.data.as_ref()).unwrap();
    config_account.token_amount_delegated = 100;

    let updated_config = Account {
        lamports: account.lamports,
        data: config_account.try_to_vec().unwrap(),
        owner: account.owner,
        executable: account.executable,
        rent_epoch: account.rent_epoch,
    };
    context.set_account(&config, &updated_config.into());

    // And a stake account (amount = 100).

    let validator = Pubkey::new_unique();
    let authority = Keypair::new();
    let vote = create_vote_account(&mut context, &validator, &authority.pubkey()).await;

    let stake_pda = create_stake(&mut context, &validator, &vote, &config).await;

    let account = get_account!(context, stake_pda);
    let mut stake_account = Stake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the stake values
    stake_account.amount = 100;

    let updated_stake = Account {
        lamports: account.lamports,
        data: stake_account.try_to_vec().unwrap(),
        owner: account.owner,
        executable: account.executable,
        rent_epoch: account.rent_epoch,
    };
    context.set_account(&stake_pda, &updated_stake.into());

    // And we create a second config.

    let wrong_config = create_config(&mut context).await;

    // When we try to inactivate the stake with the wrong config account.

    let inactivate_ix = InactivateStakeBuilder::new()
        .config(wrong_config) // <- wrong config
        .stake(stake_pda)
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
async fn fail_inactivate_stake_with_uninitialized_stake_account() {
    let mut context = ProgramTest::new("stake_program", paladin_stake::ID, None)
        .start_with_context()
        .await;

    // Given a config account and a validator's vote account.

    let config = create_config(&mut context).await;
    let validator = Pubkey::new_unique();

    // And an uninitialized stake account.

    let (stake_pda, _) = find_stake_pda(&validator, &config);

    context.set_account(
        &stake_pda,
        &AccountSharedData::from(Account {
            lamports: 100_000_000,
            data: vec![5; std::mem::size_of::<Stake>()],
            owner: paladin_stake::ID,
            ..Default::default()
        }),
    );

    // When we try to deactivate from an uninitialized stake account.

    let inactivate_ix = InactivateStakeBuilder::new()
        .config(config)
        .stake(stake_pda)
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
