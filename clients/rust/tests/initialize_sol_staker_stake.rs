#![cfg(feature = "test-sbf")]

mod setup;

use borsh::BorshSerialize;
use paladin_stake_program_client::{
    accounts::{Config, SolStakerStake, ValidatorStake},
    instructions::InitializeSolStakerStakeBuilder,
    pdas::find_sol_staker_stake_pda,
};
use setup::{
    config::ConfigManager,
    stake::{create_stake_account, delegate_stake_account},
    validator_stake::ValidatorStakeManager,
};
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{
    account::{Account, AccountSharedData},
    instruction::InstructionError,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    stake::{
        self,
        state::{Authorized, Lockup},
    },
    system_instruction,
    transaction::Transaction,
};

#[tokio::test]
async fn initialize_sol_staker_stake() {
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
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;

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
    delegate_stake_account(&mut context, &stake_state, &stake_manager.vote, &withdrawer).await;

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
        .validator_stake(stake_manager.stake)
        .sol_staker_native_stake(stake_state)
        .sol_stake_view_program(paladin_sol_stake_view_program_client::ID)
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[transfer_ix, initialize_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Assert - Key properties about the new account.
    let account = get_account!(context, stake_pda);
    assert_eq!(account.data.len(), SolStakerStake::LEN);
    let account_data = account.data.as_ref();
    let stake_account = SolStakerStake::from_bytes(account_data).unwrap();
    assert_eq!(stake_account.delegation.validator_vote, stake_manager.vote);
    assert_eq!(stake_account.delegation.authority, withdrawer.pubkey());
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
    assert_eq!(stake_account.sol_stake, stake_state);
    assert_eq!(stake_account.lamports_amount, 1_000_000_000);

    // And the validator stake account was updated.
    let account = get_account!(context, stake_manager.stake);
    let validator_stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        validator_stake_account.total_staked_lamports_amount,
        1_000_000_000
    );
}

#[tokio::test]
async fn initialize_sol_staker_stake_sets_last_seen() {
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
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;

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
    delegate_stake_account(&mut context, &stake_state, &stake_manager.vote, &withdrawer).await;

    // Set the accumulated holder & stake rewards per token.
    let mut config = get_account!(context, config_manager.config);
    let mut config_state = Config::from_bytes(&mut config.data).unwrap();
    config_state.accumulated_holder_rewards_per_token = 100;
    config_state.accumulated_stake_rewards_per_token = 250;
    config.data = config_state.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &config.into());

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
        .validator_stake(stake_manager.stake)
        .sol_staker_native_stake(stake_state)
        .sol_stake_view_program(paladin_sol_stake_view_program_client::ID)
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[transfer_ix, initialize_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Assert - Key properties about the new account.
    let account = get_account!(context, stake_pda);
    assert_eq!(account.data.len(), SolStakerStake::LEN);
    let account_data = account.data.as_ref();
    let stake_account = SolStakerStake::from_bytes(account_data).unwrap();
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

    // And the validator stake account was updated.
    let account = get_account!(context, stake_manager.stake);
    let validator_stake_account = ValidatorStake::from_bytes(account.data.as_ref()).unwrap();
    assert_eq!(
        validator_stake_account.total_staked_lamports_amount,
        1_000_000_000
    );
}

#[tokio::test]
async fn fail_initialize_sol_staker_stake_with_initialized_account() {
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
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;

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
    delegate_stake_account(&mut context, &stake_state, &stake_manager.vote, &withdrawer).await;

    // And we initialize the SOL staker stake account.

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
        .validator_stake(stake_manager.stake)
        .sol_staker_native_stake(stake_state)
        .sol_stake_view_program(paladin_sol_stake_view_program_client::ID)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[transfer_ix, initialize_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // When we try to initialize the SOL staker stake account again.

    context.get_new_latest_blockhash().await.unwrap();

    let initialize_ix = InitializeSolStakerStakeBuilder::new()
        .config(config_manager.config)
        .sol_staker_stake(stake_pda)
        .validator_stake(stake_manager.stake)
        .sol_staker_native_stake(stake_state)
        .sol_stake_view_program(paladin_sol_stake_view_program_client::ID)
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
async fn fail_initialize_sol_staker_stake_with_invalid_derivation() {
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
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;

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
    delegate_stake_account(&mut context, &stake_state, &stake_manager.vote, &withdrawer).await;

    // When we try to initialize the SOL staker stake account with the wrong derivation
    // (different address as the stake state account).

    let (stake_pda, _) = find_sol_staker_stake_pda(&Pubkey::new_unique(), &config_manager.config);

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
        .validator_stake(stake_manager.stake)
        .sol_staker_native_stake(stake_state)
        .sol_stake_view_program(paladin_sol_stake_view_program_client::ID)
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

    // Then we expect an error.

    assert_instruction_error!(err, InstructionError::InvalidSeeds);
}

#[tokio::test]
async fn fail_initialize_stake_with_invalid_stake_state() {
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
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    // And we create an invalid SOL stake account.

    let fake_stake_state = Keypair::new().pubkey();
    context.set_account(
        &fake_stake_state,
        &AccountSharedData::from(Account {
            lamports: 100_000_000,
            owner: stake::program::ID,
            ..Default::default()
        }),
    );

    // When we try initialize the SOL staker stake account with an invalid stake state account.

    let (stake_pda, _) = find_sol_staker_stake_pda(&fake_stake_state, &config_manager.config);

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
        .validator_stake(stake_manager.stake)
        .sol_staker_native_stake(fake_stake_state)
        .sol_stake_view_program(paladin_sol_stake_view_program_client::ID)
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

    // Then we expect an error.

    assert_instruction_error!(err, InstructionError::BorshIoError(..));
}

#[tokio::test]
async fn fail_initialize_sol_staker_stake_with_uninitialized_config() {
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
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;

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
    delegate_stake_account(&mut context, &stake_state, &stake_manager.vote, &withdrawer).await;

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

    // When we try initialize the SOL staker stake account with an uninitialized config account.

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
        .validator_stake(stake_manager.stake)
        .sol_staker_native_stake(stake_state)
        .sol_stake_view_program(paladin_sol_stake_view_program_client::ID)
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

    // Then we expect an error.

    assert_instruction_error!(err, InstructionError::UninitializedAccount);
}

#[tokio::test]
async fn fail_initialize_sol_staker_stake_with_invalid_sol_stake_view_program() {
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

    let fake_sol_stake_view_program = Pubkey::new_unique();
    program_test.add_program(
        "paladin_sol_stake_view_program",
        fake_sol_stake_view_program,
        None,
    );
    let mut context = program_test.start_with_context().await;

    // Given a config and a validator stake accounts.

    let config_manager = ConfigManager::new(&mut context).await;
    let stake_manager = ValidatorStakeManager::new(&mut context, &config_manager.config).await;

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
    delegate_stake_account(&mut context, &stake_state, &stake_manager.vote, &withdrawer).await;

    // When we try initialize the SOL staker stake account with an invalid SOL stake view program.

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
        .validator_stake(stake_manager.stake)
        .sol_staker_native_stake(stake_state)
        .sol_stake_view_program(fake_sol_stake_view_program) // <-- Invalid program
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

    // Then we expect an error.

    assert_instruction_error!(err, InstructionError::IncorrectProgramId);
}
