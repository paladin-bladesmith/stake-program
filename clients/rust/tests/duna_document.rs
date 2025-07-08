#![cfg(feature = "test-sbf")]

mod setup;

use paladin_stake_program::state::find_duna_document_pda;
use paladin_stake_program_client::{
    accounts::{SolStakerStake, ValidatorStake},
    instructions::{InitializeSolStakerStakeBuilder, InitializeValidatorStakeBuilder},
    pdas::{
        find_sol_staker_authority_override_pda, find_sol_staker_stake_pda, find_validator_stake_pda,
    },
};
use setup::{config::create_config, vote::create_vote_account};
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{
    account::{Account, AccountSharedData},
    instruction::InstructionError,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    stake::state::{Authorized, Lockup},
    system_instruction,
    transaction::Transaction,
};

use crate::setup::{
    config::{get_duna_hash, ConfigManager},
    stake::{create_stake_account, delegate_stake_account},
    validator_stake::ValidatorStakeManager,
    DUNA_PROGRAM_ID,
};

#[tokio::test]
async fn fail_initialize_validator_stake_unintialized_duna_pda() {
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

    // Set duna document PDA uninitialized.
    let (duna_document_pda, _) = find_duna_document_pda(&validator, &get_duna_hash());

    context.set_account(
        &duna_document_pda,
        &AccountSharedData::from(Account {
            lamports: 100_000_000,
            data: vec![0; 1],
            owner: DUNA_PROGRAM_ID,
            ..Default::default()
        }),
    );

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
        .duna_document_pda(duna_document_pda)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[transfer_ix, initialize_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_instruction_error!(err, InstructionError::Custom(22));
}

#[tokio::test]
async fn fail_initialize_sol_staker_unintialized_duna_pda() {
    let mut program_test = ProgramTest::new(
        "paladin_stake_program",
        paladin_stake_program_client::ID,
        None,
    );
    program_test.add_program(
        "paladin_sol_stake_view_program",
        paladin_sol_stake_view_program_client::ID,
        None,
    );
    let mut context = program_test.start_with_context().await;

    // Given a config and a validator stake accounts.
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    // And we create a SOL stake account.
    let stake_state = Keypair::new();
    let stake_amount = 1_000_000_000;
    let withdrawer = Keypair::new();
    create_stake_account(
        &mut context,
        &stake_state,
        &Authorized::auto(&withdrawer.pubkey()),
        &Lockup::default(),
        stake_amount,
    )
    .await;
    let stake_state = stake_state.pubkey();
    delegate_stake_account(
        &mut context,
        &stake_state,
        &validator_stake_manager.vote,
        &withdrawer,
    )
    .await;

    // Sign duna document PDA.
    let (duna_document_pda, _) = find_duna_document_pda(&withdrawer.pubkey(), &get_duna_hash());

    context.set_account(
        &duna_document_pda,
        &AccountSharedData::from(Account {
            lamports: 100_000_000,
            data: vec![0; 1],
            owner: DUNA_PROGRAM_ID,
            ..Default::default()
        }),
    );

    // When we initialize the SOL staker stake account.
    let (stake_pda, _) = find_sol_staker_stake_pda(&stake_state, &config_manager.config);
    let transfer_ix = system_instruction::transfer(
        &context.payer.pubkey(),
        &stake_pda,
        context
            .banks_client
            .get_rent()
            .await
            .unwrap()
            .minimum_balance(SolStakerStake::LEN),
    );
    let initialize_ix = InitializeSolStakerStakeBuilder::new()
        .config(config_manager.config)
        .sol_staker_stake(stake_pda)
        .sol_staker_authority_override(
            find_sol_staker_authority_override_pda(&withdrawer.pubkey(), &config_manager.config).0,
        )
        .validator_stake(validator_stake_manager.stake)
        .sol_staker_native_stake(stake_state)
        .sol_stake_view_program(paladin_sol_stake_view_program_client::ID)
        .duna_document_pda(duna_document_pda)
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[transfer_ix, initialize_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_instruction_error!(err, InstructionError::Custom(22));
}
