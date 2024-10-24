#![cfg(feature = "test-sbf")]

mod setup;

use borsh::BorshSerialize;
use paladin_stake_program_client::{
    accounts::{Config, ValidatorStake},
    instructions::InitializeValidatorStakeBuilder,
    pdas::find_validator_stake_pda,
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
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config account and a validator's vote account.
    let config = create_config(&mut context).await;
    let validator = Pubkey::new_unique();
    let validator_vote = create_vote_account(&mut context, &validator, &validator).await;

    // When we initialize the stake account.
    let (stake_pda, _) = find_validator_stake_pda(&validator_vote, &config);
    let transfer_ix = system_instruction::transfer(
        &context.payer.pubkey(),
        &stake_pda,
        context
            .banks_client
            .get_rent()
            .await
            .unwrap()
            .minimum_balance(ValidatorStake::LEN),
    );

    let initialize_ix = InitializeValidatorStakeBuilder::new()
        .config(config)
        .validator_stake(stake_pda)
        .validator_vote(validator_vote)
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
    assert_eq!(account.data.len(), ValidatorStake::LEN);
    let account_data = account.data.as_ref();
    let stake_account = ValidatorStake::from_bytes(account_data).unwrap();
    assert_eq!(stake_account.delegation.validator_vote, validator_vote);
    assert_eq!(stake_account.delegation.authority, validator);
    assert_eq!(stake_account.delegation.active_amount, 0);
    assert_eq!(stake_account.delegation.inactive_amount, 0);
    assert_eq!(stake_account.delegation.effective_amount, 0);
    assert_eq!(
        stake_account.delegation.last_seen_holder_rewards_per_token,
        0
    );
    assert_eq!(
        stake_account.delegation.last_seen_stake_rewards_per_token,
        0
    );
}

#[tokio::test]
async fn initialize_stake_with_validator_vote_sets_last_seen() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config account and a validator's vote account.
    let config = create_config(&mut context).await;
    let validator = Pubkey::new_unique();
    let validator_vote = create_vote_account(&mut context, &validator, &validator).await;

    // Set the accumulated holder & stake rewards per token.
    let mut config_account = get_account!(context, config);
    let mut config_state = Config::from_bytes(&mut config_account.data).unwrap();
    config_state.accumulated_holder_rewards_per_token = 100;
    config_state.accumulated_stake_rewards_per_token = 250;
    config_account.data = config_state.try_to_vec().unwrap();
    context.set_account(&config, &config_account.into());

    // When we initialize the stake account.
    let (stake_pda, _) = find_validator_stake_pda(&validator_vote, &config);
    let transfer_ix = system_instruction::transfer(
        &context.payer.pubkey(),
        &stake_pda,
        context
            .banks_client
            .get_rent()
            .await
            .unwrap()
            .minimum_balance(ValidatorStake::LEN),
    );

    let initialize_ix = InitializeValidatorStakeBuilder::new()
        .config(config)
        .validator_stake(stake_pda)
        .validator_vote(validator_vote)
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
    assert_eq!(account.data.len(), ValidatorStake::LEN);
    let account_data = account.data.as_ref();
    let stake_account = ValidatorStake::from_bytes(account_data).unwrap();
    assert_eq!(stake_account.delegation.validator_vote, validator_vote);
    assert_eq!(stake_account.delegation.authority, validator);
    assert_eq!(stake_account.delegation.active_amount, 0);
    assert_eq!(stake_account.delegation.inactive_amount, 0);
    assert_eq!(stake_account.delegation.effective_amount, 0);
    assert_eq!(
        stake_account.delegation.last_seen_holder_rewards_per_token,
        100
    );
    assert_eq!(
        stake_account.delegation.last_seen_stake_rewards_per_token,
        250
    );
}

#[tokio::test]
async fn fail_initialize_stake_with_initialized_account() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config account and a validator's vote account.

    let config = create_config(&mut context).await;
    let validator = Pubkey::new_unique();
    let validator_vote = create_vote_account(&mut context, &validator, &validator).await;

    // And we initialize the stake account.

    let (stake_pda, _) = find_validator_stake_pda(&validator_vote, &config);

    let transfer_ix = system_instruction::transfer(
        &context.payer.pubkey(),
        &stake_pda,
        context
            .banks_client
            .get_rent()
            .await
            .unwrap()
            .minimum_balance(ValidatorStake::LEN),
    );

    let initialize_ix = InitializeValidatorStakeBuilder::new()
        .config(config)
        .validator_stake(stake_pda)
        .validator_vote(validator_vote)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[transfer_ix, initialize_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    let account = get_account!(context, stake_pda);
    assert_eq!(account.data.len(), ValidatorStake::LEN);

    // When we try to initialize the stake account again.

    let initialize_ix = InitializeValidatorStakeBuilder::new()
        .config(config)
        .validator_stake(stake_pda)
        .validator_vote(validator_vote)
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
async fn fail_initialize_stake_with_invalid_derivation() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config account and a validator's vote account.

    let config = create_config(&mut context).await;
    let validator = Pubkey::new_unique();
    let validator_vote = create_vote_account(&mut context, &validator, &validator).await;

    // When we try initialize the stake account with an invalid derivation.

    let (stake_pda, _) = find_validator_stake_pda(&Pubkey::new_unique(), &config);

    let initialize_ix = InitializeValidatorStakeBuilder::new()
        .config(config)
        .validator_stake(stake_pda)
        .validator_vote(validator_vote)
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
async fn fail_initialize_stake_with_invalid_vote_account() {
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a config account and a validator's vote account.

    let config = create_config(&mut context).await;
    let validator = Pubkey::new_unique();
    let validator_vote = create_vote_account_with_program_id(
        &mut context,
        &validator,
        &validator,
        &system_program::ID,
    )
    .await;

    // When we try initialize the stake account with an invalid validator vote account
    // (the validator vote account is owned by system program).

    let (stake_pda, _) = find_validator_stake_pda(&Pubkey::new_unique(), &config);

    let initialize_ix = InitializeValidatorStakeBuilder::new()
        .config(config)
        .validator_stake(stake_pda)
        .validator_vote(validator_vote)
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
    let mut context = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    )
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
        &paladin_stake_program_client::ID,
    );

    let validator = Pubkey::new_unique();
    let validator_vote = create_vote_account_with_program_id(
        &mut context,
        &validator,
        &validator,
        &system_program::ID,
    )
    .await;

    // When we try initialize the stake account with an invalid derivation.

    let (stake_pda, _) =
        find_validator_stake_pda(&Pubkey::new_unique(), &uninitialized_config.pubkey());

    let initialize_ix = InitializeValidatorStakeBuilder::new()
        .config(uninitialized_config.pubkey())
        .validator_stake(stake_pda)
        .validator_vote(validator_vote)
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
