#![cfg(feature = "test-sbf")]

mod setup;

use paladin_stake::{
    accounts::{Config, Stake},
    instructions::InitializeStakeBuilder,
    pdas::find_stake_pda,
};
use setup::{
    config::create_config,
    vote::{create_vote_account, create_vote_account_with_program_id},
};
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{
    instruction::InstructionError,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction, system_program,
    transaction::Transaction,
};

#[tokio::test]
async fn initialize_stake_with_validator_vote() {
    let mut context = ProgramTest::new("stake_program", paladin_stake::ID, None)
        .start_with_context()
        .await;

    // Given a config account and a validator's vote account.

    let config = create_config(&mut context).await;
    let validator = Pubkey::new_unique();
    let vote = create_vote_account(&mut context, &validator, &validator);

    // When we initialize the stake account.

    let (stake_pda, _) = find_stake_pda(&validator, &config);

    let transfer_ix = system_instruction::transfer(
        &context.payer.pubkey(),
        &stake_pda,
        context
            .banks_client
            .get_rent()
            .await
            .unwrap()
            .minimum_balance(Stake::LEN),
    );

    let initialize_ix = InitializeStakeBuilder::new()
        .config(config)
        .stake(stake_pda)
        .validator_vote(vote)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[transfer_ix, initialize_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then an account was created with the correct data.

    let account = get_account!(context, stake_pda);
    assert_eq!(account.data.len(), Stake::LEN);

    let account_data = account.data.as_ref();
    let stake_account = Stake::from_bytes(account_data).unwrap();
    assert_eq!(stake_account.validator, validator);
    assert_eq!(stake_account.authority, validator);
}

#[tokio::test]
async fn fail_initialize_stake_with_initialized_account() {
    let mut context = ProgramTest::new("stake_program", paladin_stake::ID, None)
        .start_with_context()
        .await;

    // Given a config account and a validator's vote account.

    let config = create_config(&mut context).await;
    let validator = Pubkey::new_unique();
    let vote = create_vote_account(&mut context, &validator, &validator);

    // And we initialize the stake account.

    let (stake_pda, _) = find_stake_pda(&validator, &config);

    let transfer_ix = system_instruction::transfer(
        &context.payer.pubkey(),
        &stake_pda,
        context
            .banks_client
            .get_rent()
            .await
            .unwrap()
            .minimum_balance(Stake::LEN),
    );

    let initialize_ix = InitializeStakeBuilder::new()
        .config(config)
        .stake(stake_pda)
        .validator_vote(vote)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[transfer_ix, initialize_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    let account = get_account!(context, stake_pda);
    assert_eq!(account.data.len(), Stake::LEN);

    // When we try to initialize the stake account again.

    let initialize_ix = InitializeStakeBuilder::new()
        .config(config)
        .stake(stake_pda)
        .validator_vote(vote)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[initialize_ix],
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

    assert_instruction_error!(err, InstructionError::AccountAlreadyInitialized);
}

#[tokio::test]
async fn fail_initialize_stake_without_funding_account() {
    let mut context = ProgramTest::new("stake_program", paladin_stake::ID, None)
        .start_with_context()
        .await;

    // Given a config account and a validator's vote account.

    let config = create_config(&mut context).await;
    let validator = Pubkey::new_unique();
    let vote = create_vote_account(&mut context, &validator, &validator);

    // And we initialize the stake account without pre-funding the account.

    let (stake_pda, _) = find_stake_pda(&validator, &config);

    let initialize_ix = InitializeStakeBuilder::new()
        .config(config)
        .stake(stake_pda)
        .validator_vote(vote)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[initialize_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    let account = context.banks_client.get_account(stake_pda).await.unwrap();
    // TODO: Shouldn't we get an error from the instruction?
    assert!(account.is_none());
}

#[tokio::test]
async fn fail_initialize_stake_with_invalid_stake_account() {
    let mut context = ProgramTest::new("stake_program", paladin_stake::ID, None)
        .start_with_context()
        .await;

    // Given a config account and a validator's vote account.

    let config = create_config(&mut context).await;
    let validator = Pubkey::new_unique();
    let vote = create_vote_account(&mut context, &validator, &validator);

    // When we try to initialize the stake account with an invalid PDA.

    let stake_pda = Pubkey::new_unique();

    let initialize_ix = InitializeStakeBuilder::new()
        .config(config)
        .stake(stake_pda)
        .validator_vote(vote)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[initialize_ix],
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
async fn fail_initialize_stake_with_invalid_derivation() {
    let mut context = ProgramTest::new("stake_program", paladin_stake::ID, None)
        .start_with_context()
        .await;

    // Given a config account and a validator's vote account.

    let config = create_config(&mut context).await;
    let validator = Pubkey::new_unique();
    let vote = create_vote_account(&mut context, &validator, &validator);

    // When we try initialize the stake account with an invalid derivation.

    let (stake_pda, _) = find_stake_pda(&Pubkey::new_unique(), &config);

    let initialize_ix = InitializeStakeBuilder::new()
        .config(config)
        .stake(stake_pda)
        .validator_vote(vote)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[initialize_ix],
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
async fn fail_initialize_stake_with_invalid_vote_account_owner() {
    let mut context = ProgramTest::new("stake_program", paladin_stake::ID, None)
        .start_with_context()
        .await;

    // Given a config account and a validator's vote account.

    let config = create_config(&mut context).await;
    let validator = Pubkey::new_unique();
    let vote = create_vote_account_with_program_id(
        &mut context,
        &validator,
        &validator,
        &system_program::ID,
    );

    // When we try initialize the stake account with an invalid derivation.

    let (stake_pda, _) = find_stake_pda(&Pubkey::new_unique(), &config);

    let initialize_ix = InitializeStakeBuilder::new()
        .config(config)
        .stake(stake_pda)
        .validator_vote(vote)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[initialize_ix],
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

    assert_instruction_error!(err, InstructionError::InvalidAccountOwner);
}

#[tokio::test]
async fn fail_initialize_stake_with_uninitialized_config_account() {
    let mut context = ProgramTest::new("stake_program", paladin_stake::ID, None)
        .start_with_context()
        .await;

    // Given a config account and a validator's vote account.

    let uninitialized_config = Keypair::new();

    let create_config_ix = system_instruction::create_account(
        &context.payer.pubkey(),
        &uninitialized_config.pubkey(),
        context
            .banks_client
            .get_rent()
            .await
            .unwrap()
            .minimum_balance(Config::LEN),
        Config::LEN as u64,
        &paladin_stake::ID,
    );

    let validator = Pubkey::new_unique();
    let vote = create_vote_account_with_program_id(
        &mut context,
        &validator,
        &validator,
        &system_program::ID,
    );

    // When we try initialize the stake account with an invalid derivation.

    let (stake_pda, _) = find_stake_pda(&Pubkey::new_unique(), &uninitialized_config.pubkey());

    let initialize_ix = InitializeStakeBuilder::new()
        .config(uninitialized_config.pubkey())
        .stake(stake_pda)
        .validator_vote(vote)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[create_config_ix, initialize_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &uninitialized_config],
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
