#![cfg(feature = "test-sbf")]

mod setup;

use borsh::BorshSerialize;
use paladin_stake::{
    accounts::Stake, errors::StakeError, instructions::DeactivateStakeBuilder, pdas::find_stake_pda,
};
use setup::{config::create_config, stake::create_stake, vote::create_vote_account};
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{
    account::{Account, AccountSharedData},
    instruction::InstructionError,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

#[tokio::test]
async fn deactivate_stake() {
    let mut context = ProgramTest::new("stake_program", paladin_stake::ID, None)
        .start_with_context()
        .await;

    // Given a config account and a validator's vote account.

    let config = create_config(&mut context).await;
    let validator = Pubkey::new_unique();
    let authority = Keypair::new();
    let vote = create_vote_account(&mut context, &validator, &authority.pubkey()).await;

    // And a stake account.

    let stake_pda = create_stake(&mut context, &validator, &vote, &config).await;

    let account = get_account!(context, stake_pda);
    let mut stake_account = Stake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the amount to 100
    stake_account.amount = 100;

    let updated_stake = Account {
        lamports: account.lamports,
        data: stake_account.try_to_vec().unwrap(),
        owner: account.owner,
        executable: account.executable,
        rent_epoch: account.rent_epoch,
    };

    context.set_account(&stake_pda, &updated_stake.into());

    // When we deactivate an amount from the stake account.

    let deactivate_ix = DeactivateStakeBuilder::new()
        .stake(stake_pda)
        .stake_authority(authority.pubkey())
        .args(50)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[deactivate_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the deactivation should be successful.

    let account = get_account!(context, stake_pda);
    let stake_account = Stake::from_bytes(account.data.as_ref()).unwrap();

    assert_eq!(stake_account.amount, 100);
    assert_eq!(stake_account.deactivating_amount, 50);
    assert!(stake_account.deactivation_timestamp.value().is_some())
}

#[tokio::test]
async fn deactivate_stake_with_active_deactivation() {
    let mut context = ProgramTest::new("stake_program", paladin_stake::ID, None)
        .start_with_context()
        .await;

    // Given a config account and a validator's vote account.

    let config = create_config(&mut context).await;
    let validator = Pubkey::new_unique();
    let authority = Keypair::new();
    let vote = create_vote_account(&mut context, &validator, &authority.pubkey()).await;

    // And a stake account.

    let stake_pda = create_stake(&mut context, &validator, &vote, &config).await;

    let account = get_account!(context, stake_pda);
    let mut stake_account = Stake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the amount to 100
    stake_account.amount = 100;

    let updated_stake = Account {
        lamports: account.lamports,
        data: stake_account.try_to_vec().unwrap(),
        owner: account.owner,
        executable: account.executable,
        rent_epoch: account.rent_epoch,
    };

    context.set_account(&stake_pda, &updated_stake.into());

    // And we deactivate an amount from the stake account.

    let deactivate_ix = DeactivateStakeBuilder::new()
        .stake(stake_pda)
        .stake_authority(authority.pubkey())
        .args(50)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[deactivate_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    let account = get_account!(context, stake_pda);
    let stake_account = Stake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake_account.deactivating_amount, 50);

    // When we deactivate a different amount from the stake account
    // with an active deactivation.

    let deactivate_ix = DeactivateStakeBuilder::new()
        .stake(stake_pda)
        .stake_authority(authority.pubkey())
        .args(100)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[deactivate_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then the deactivation should be "updated".

    let account = get_account!(context, stake_pda);
    let stake_account = Stake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(stake_account.deactivating_amount, 100);
}

#[tokio::test]
async fn fail_deactivate_stake_with_amount_greater_than_stake_amount() {
    let mut context = ProgramTest::new("stake_program", paladin_stake::ID, None)
        .start_with_context()
        .await;

    // Given a config account and a validator's vote account.

    let config = create_config(&mut context).await;
    let validator = Pubkey::new_unique();
    let authority = Keypair::new();
    let vote = create_vote_account(&mut context, &validator, &authority.pubkey()).await;

    // And a stake account.

    let stake_pda = create_stake(&mut context, &validator, &vote, &config).await;

    let account = get_account!(context, stake_pda);
    let mut stake_account = Stake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the amount to 100
    stake_account.amount = 100;

    let updated_stake = Account {
        lamports: account.lamports,
        data: stake_account.try_to_vec().unwrap(),
        owner: account.owner,
        executable: account.executable,
        rent_epoch: account.rent_epoch,
    };

    context.set_account(&stake_pda, &updated_stake.into());

    // When we try to deactivate an amount greater than the staked amount.

    let deactivate_ix = DeactivateStakeBuilder::new()
        .stake(stake_pda)
        .stake_authority(authority.pubkey())
        .args(150)
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

    assert_custom_error!(err, StakeError::InsufficientStakeAmount);
}

#[tokio::test]
async fn fail_deactivate_stake_with_invalid_authority() {
    let mut context = ProgramTest::new("stake_program", paladin_stake::ID, None)
        .start_with_context()
        .await;

    // Given a config account and a validator's vote account.

    let config = create_config(&mut context).await;
    let validator = Pubkey::new_unique();
    let authority = Keypair::new();
    let vote = create_vote_account(&mut context, &validator, &authority.pubkey()).await;

    // And a stake account.

    let stake_pda = create_stake(&mut context, &validator, &vote, &config).await;

    let account = get_account!(context, stake_pda);
    let mut stake_account = Stake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the amount to 100
    stake_account.amount = 100;

    let updated_stake = Account {
        lamports: account.lamports,
        data: stake_account.try_to_vec().unwrap(),
        owner: account.owner,
        executable: account.executable,
        rent_epoch: account.rent_epoch,
    };

    context.set_account(&stake_pda, &updated_stake.into());

    // When we try to deactivate with an invalid authority.

    let fake_authority = Keypair::new();

    let deactivate_ix = DeactivateStakeBuilder::new()
        .stake(stake_pda)
        .stake_authority(fake_authority.pubkey())
        .args(50)
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

    assert_custom_error!(err, StakeError::InvalidAuthority);
}

#[tokio::test]
async fn fail_deactivate_stake_with_zero_amount() {
    let mut context = ProgramTest::new("stake_program", paladin_stake::ID, None)
        .start_with_context()
        .await;

    // Given a config account and a validator's vote account.

    let config = create_config(&mut context).await;
    let validator = Pubkey::new_unique();
    let authority = Keypair::new();
    let vote = create_vote_account(&mut context, &validator, &authority.pubkey()).await;

    // And a stake account.

    let stake_pda = create_stake(&mut context, &validator, &vote, &config).await;

    let account = get_account!(context, stake_pda);
    let mut stake_account = Stake::from_bytes(account.data.as_ref()).unwrap();
    // "manually" set the amount to 100
    stake_account.amount = 100;

    let updated_stake = Account {
        lamports: account.lamports,
        data: stake_account.try_to_vec().unwrap(),
        owner: account.owner,
        executable: account.executable,
        rent_epoch: account.rent_epoch,
    };

    context.set_account(&stake_pda, &updated_stake.into());

    // When we try to deactivate with an invalid amount (0).

    let deactivate_ix = DeactivateStakeBuilder::new()
        .stake(stake_pda)
        .stake_authority(authority.pubkey())
        .args(0)
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

    assert_custom_error!(err, StakeError::InvalidAmount);
}

#[tokio::test]
async fn fail_deactivate_stake_with_uninitialized_stake_account() {
    let mut context = ProgramTest::new("stake_program", paladin_stake::ID, None)
        .start_with_context()
        .await;

    // Given a config account and a validator's vote account.

    let config = create_config(&mut context).await;
    let validator = Pubkey::new_unique();
    let authority = Keypair::new();

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

    let deactivate_ix = DeactivateStakeBuilder::new()
        .stake(stake_pda)
        .stake_authority(authority.pubkey())
        .args(0)
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
