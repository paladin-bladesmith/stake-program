#![cfg(feature = "test-sbf")]

use borsh::BorshSerialize;
use paladin_stake_program_client::accounts::Config;
use paladin_stake_program_client::errors::PaladinStakeProgramError;
use paladin_stake_program_client::instructions::SolStakerMoveTokensInstructionArgs;
use paladin_stake_program_client::{accounts::SolStakerStake, instructions::SolStakerMoveTokens};
use setup::validator_stake::ValidatorStakeManager;
use setup::{config::ConfigManager, sol_staker_stake::SolStakerStakeManager};
use solana_program_test::tokio;
use solana_sdk::instruction::InstructionError;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;

mod setup;

#[tokio::test]
async fn transfer_to_empty() {
    let mut context = setup::setup(&[]).await;

    // Setup the relevant accounts.
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let stake_authority = Keypair::new();
    let source_sol_staker_staker_manager = SolStakerStakeManager::new_with_authority(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        stake_authority.insecure_clone(),
        5_000_000_000, // 5 SOL staked
    )
    .await;
    let destination_sol_staker_staker_manager = SolStakerStakeManager::new_with_authority(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        stake_authority,
        5_000_000_000, // 5 SOL staked
    )
    .await;

    // Stake 10 PAL on the source account.
    let mut source = get_account!(context, source_sol_staker_staker_manager.stake);
    let mut source_state = SolStakerStake::from_bytes(&source.data).unwrap();
    source_state.delegation.staked_amount = 10;
    source.data = source_state.try_to_vec().unwrap();
    context.set_account(&source_sol_staker_staker_manager.stake, &source.into());

    // Act - Transfer 5 PAL to the destination sol staker stake.
    let sol_staker_move_tokens = SolStakerMoveTokens {
        config: config_manager.config,
        vault_holder_rewards: config_manager.vault_holder_rewards,
        sol_staker_authority: source_sol_staker_staker_manager.authority.pubkey(),
        source_sol_staker_stake: source_sol_staker_staker_manager.stake,
        destination_sol_staker_stake: destination_sol_staker_staker_manager.stake,
    }
    .instruction(SolStakerMoveTokensInstructionArgs { amount: 3 });
    let tx = Transaction::new_signed_with_payer(
        &[sol_staker_move_tokens],
        Some(&context.payer.pubkey()),
        &[&context.payer, &source_sol_staker_staker_manager.authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Assert - Source account has 5 PAL staked.
    let source = get_account!(context, source_sol_staker_staker_manager.stake);
    let source = SolStakerStake::from_bytes(&source.data).unwrap();
    assert_eq!(source.delegation.staked_amount, 7);
    assert_eq!(source.delegation.effective_amount, 7);

    // Assert - Destination account has 5 PAL staked.
    let destination = get_account!(context, destination_sol_staker_staker_manager.stake);
    let destination = SolStakerStake::from_bytes(&destination.data).unwrap();
    assert_eq!(destination.delegation.staked_amount, 3);
    assert_eq!(destination.delegation.effective_amount, 3);

    // Assert - Config has 10 effective.
    let config = get_account!(context, config_manager.config);
    let config = Config::from_bytes(&config.data).unwrap();
    assert_eq!(config.accumulated_stake_rewards_per_token, 0);
    assert_eq!(config.token_amount_effective, 10);
}

#[tokio::test]
async fn transfer_to_not_empty() {
    let mut context = setup::setup(&[]).await;

    // Setup the relevant accounts.
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let stake_authority = Keypair::new();
    let source_sol_staker_staker_manager = SolStakerStakeManager::new_with_authority(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        stake_authority.insecure_clone(),
        5_000_000_000, // 5 SOL staked
    )
    .await;
    let destination_sol_staker_staker_manager = SolStakerStakeManager::new_with_authority(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        stake_authority,
        5_000_000_000, // 5 SOL staked
    )
    .await;

    // Stake 10 PAL on the source account.
    let mut source = get_account!(context, source_sol_staker_staker_manager.stake);
    let mut source_state = SolStakerStake::from_bytes(&source.data).unwrap();
    source_state.delegation.staked_amount = 10;
    source.data = source_state.try_to_vec().unwrap();
    context.set_account(&source_sol_staker_staker_manager.stake, &source.into());

    // Stake 10 PAL on the destination account.
    let mut destination = get_account!(context, destination_sol_staker_staker_manager.stake);
    let mut destination_state = SolStakerStake::from_bytes(&destination.data).unwrap();
    destination_state.delegation.staked_amount = 10;
    destination.data = destination_state.try_to_vec().unwrap();
    context.set_account(
        &destination_sol_staker_staker_manager.stake,
        &destination.into(),
    );

    // Act - Transfer 5 PAL to the destination sol staker stake.
    let sol_staker_move_tokens = SolStakerMoveTokens {
        config: config_manager.config,
        vault_holder_rewards: config_manager.vault_holder_rewards,
        sol_staker_authority: source_sol_staker_staker_manager.authority.pubkey(),
        source_sol_staker_stake: source_sol_staker_staker_manager.stake,
        destination_sol_staker_stake: destination_sol_staker_staker_manager.stake,
    }
    .instruction(SolStakerMoveTokensInstructionArgs { amount: 5 });
    let tx = Transaction::new_signed_with_payer(
        &[sol_staker_move_tokens],
        Some(&context.payer.pubkey()),
        &[&context.payer, &source_sol_staker_staker_manager.authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Assert - Source account has 5 PAL staked.
    let source = get_account!(context, source_sol_staker_staker_manager.stake);
    let source = SolStakerStake::from_bytes(&source.data).unwrap();
    assert_eq!(source.delegation.staked_amount, 5);
    assert_eq!(source.delegation.effective_amount, 5);

    // Assert - Destination account has 15 PAL staked.
    let destination = get_account!(context, destination_sol_staker_staker_manager.stake);
    let destination = SolStakerStake::from_bytes(&destination.data).unwrap();
    assert_eq!(destination.delegation.staked_amount, 15);
    assert_eq!(destination.delegation.effective_amount, 15);

    // Assert - Config has 20 effective.
    let config = get_account!(context, config_manager.config);
    let config = Config::from_bytes(&config.data).unwrap();
    assert_eq!(config.accumulated_stake_rewards_per_token, 0);
    assert_eq!(config.token_amount_effective, 20);
}

#[tokio::test]
async fn transfer_from_account_with_cooldown() {
    let mut context = setup::setup(&[]).await;

    // Setup the relevant accounts.
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let stake_authority = Keypair::new();
    let source_sol_staker_staker_manager = SolStakerStakeManager::new_with_authority(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        stake_authority.insecure_clone(),
        5_000_000_000, // 5 SOL staked
    )
    .await;
    let destination_sol_staker_staker_manager = SolStakerStakeManager::new_with_authority(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        stake_authority,
        5_000_000_000, // 5 SOL staked
    )
    .await;

    // Stake 10 PAL on the source account.
    let mut source = get_account!(context, source_sol_staker_staker_manager.stake);
    let mut source_state = SolStakerStake::from_bytes(&source.data).unwrap();
    source_state.delegation.staked_amount = 10;
    source_state.delegation.unstake_cooldown = 42;
    source.data = source_state.try_to_vec().unwrap();
    context.set_account(&source_sol_staker_staker_manager.stake, &source.into());

    // Stake 10 PAL on the destination account.
    let mut destination = get_account!(context, destination_sol_staker_staker_manager.stake);
    let mut destination_state = SolStakerStake::from_bytes(&destination.data).unwrap();
    destination_state.delegation.staked_amount = 10;
    destination.data = destination_state.try_to_vec().unwrap();
    context.set_account(
        &destination_sol_staker_staker_manager.stake,
        &destination.into(),
    );

    // Act - Transfer 5 PAL to the destination sol staker stake.
    let sol_staker_move_tokens = SolStakerMoveTokens {
        config: config_manager.config,
        vault_holder_rewards: config_manager.vault_holder_rewards,
        sol_staker_authority: source_sol_staker_staker_manager.authority.pubkey(),
        source_sol_staker_stake: source_sol_staker_staker_manager.stake,
        destination_sol_staker_stake: destination_sol_staker_staker_manager.stake,
    }
    .instruction(SolStakerMoveTokensInstructionArgs { amount: 5 });
    let tx = Transaction::new_signed_with_payer(
        &[sol_staker_move_tokens],
        Some(&context.payer.pubkey()),
        &[&context.payer, &source_sol_staker_staker_manager.authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Assert - Source account has 5 PAL staked.
    let source = get_account!(context, source_sol_staker_staker_manager.stake);
    let source = SolStakerStake::from_bytes(&source.data).unwrap();
    assert_eq!(source.delegation.staked_amount, 5);
    assert_eq!(source.delegation.effective_amount, 5);
    assert_eq!(source.delegation.unstake_cooldown, 42);

    // Assert - Destination account has 15 PAL staked.
    let destination = get_account!(context, destination_sol_staker_staker_manager.stake);
    let destination = SolStakerStake::from_bytes(&destination.data).unwrap();
    assert_eq!(destination.delegation.staked_amount, 15);
    assert_eq!(destination.delegation.effective_amount, 15);
    assert_eq!(destination.delegation.unstake_cooldown, 42);

    // Assert - Config has 20 effective.
    let config = get_account!(context, config_manager.config);
    let config = Config::from_bytes(&config.data).unwrap();
    assert_eq!(config.accumulated_stake_rewards_per_token, 0);
    assert_eq!(config.token_amount_effective, 20);
}

#[tokio::test]
async fn transfer_to_account_with_cooldown() {
    let mut context = setup::setup(&[]).await;

    // Setup the relevant accounts.
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let stake_authority = Keypair::new();
    let source_sol_staker_staker_manager = SolStakerStakeManager::new_with_authority(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        stake_authority.insecure_clone(),
        5_000_000_000, // 5 SOL staked
    )
    .await;
    let destination_sol_staker_staker_manager = SolStakerStakeManager::new_with_authority(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        stake_authority,
        5_000_000_000, // 5 SOL staked
    )
    .await;

    // Stake 10 PAL on the source account.
    let mut source = get_account!(context, source_sol_staker_staker_manager.stake);
    let mut source_state = SolStakerStake::from_bytes(&source.data).unwrap();
    source_state.delegation.staked_amount = 10;
    source.data = source_state.try_to_vec().unwrap();
    context.set_account(&source_sol_staker_staker_manager.stake, &source.into());

    // Stake 10 PAL on the destination account.
    let mut destination = get_account!(context, destination_sol_staker_staker_manager.stake);
    let mut destination_state = SolStakerStake::from_bytes(&destination.data).unwrap();
    destination_state.delegation.staked_amount = 10;
    destination_state.delegation.unstake_cooldown = 42;
    destination.data = destination_state.try_to_vec().unwrap();
    context.set_account(
        &destination_sol_staker_staker_manager.stake,
        &destination.into(),
    );

    // Act - Transfer 5 PAL to the destination sol staker stake.
    let sol_staker_move_tokens = SolStakerMoveTokens {
        config: config_manager.config,
        vault_holder_rewards: config_manager.vault_holder_rewards,
        sol_staker_authority: source_sol_staker_staker_manager.authority.pubkey(),
        source_sol_staker_stake: source_sol_staker_staker_manager.stake,
        destination_sol_staker_stake: destination_sol_staker_staker_manager.stake,
    }
    .instruction(SolStakerMoveTokensInstructionArgs { amount: 5 });
    let tx = Transaction::new_signed_with_payer(
        &[sol_staker_move_tokens],
        Some(&context.payer.pubkey()),
        &[&context.payer, &source_sol_staker_staker_manager.authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Assert - Source account has 5 PAL staked.
    let source = get_account!(context, source_sol_staker_staker_manager.stake);
    let source = SolStakerStake::from_bytes(&source.data).unwrap();
    assert_eq!(source.delegation.staked_amount, 5);
    assert_eq!(source.delegation.effective_amount, 5);
    assert_eq!(source.delegation.unstake_cooldown, 0);

    // Assert - Destination account has 15 PAL staked.
    let destination = get_account!(context, destination_sol_staker_staker_manager.stake);
    let destination = SolStakerStake::from_bytes(&destination.data).unwrap();
    assert_eq!(destination.delegation.staked_amount, 15);
    assert_eq!(destination.delegation.effective_amount, 15);
    assert_eq!(destination.delegation.unstake_cooldown, 42);

    // Assert - Config has 20 effective.
    let config = get_account!(context, config_manager.config);
    let config = Config::from_bytes(&config.data).unwrap();
    assert_eq!(config.accumulated_stake_rewards_per_token, 0);
    assert_eq!(config.token_amount_effective, 20);
}

#[tokio::test]
async fn transfer_without_authority_signature_err() {
    let mut context = setup::setup(&[]).await;

    // Setup the relevant accounts.
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let stake_authority = Keypair::new();
    let source_sol_staker_staker_manager = SolStakerStakeManager::new_with_authority(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        stake_authority.insecure_clone(),
        5_000_000_000, // 5 SOL staked
    )
    .await;
    let destination_sol_staker_staker_manager = SolStakerStakeManager::new_with_authority(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        stake_authority,
        5_000_000_000, // 5 SOL staked
    )
    .await;

    // Stake 10 PAL on the source account.
    let mut source = get_account!(context, source_sol_staker_staker_manager.stake);
    let mut source_state = SolStakerStake::from_bytes(&source.data).unwrap();
    source_state.delegation.staked_amount = 10;
    source.data = source_state.try_to_vec().unwrap();
    context.set_account(&source_sol_staker_staker_manager.stake, &source.into());

    // Stake 10 PAL on the destination account.
    let mut destination = get_account!(context, destination_sol_staker_staker_manager.stake);
    let mut destination_state = SolStakerStake::from_bytes(&destination.data).unwrap();
    destination_state.delegation.staked_amount = 10;
    destination.data = destination_state.try_to_vec().unwrap();
    context.set_account(
        &destination_sol_staker_staker_manager.stake,
        &destination.into(),
    );

    // Act - Transfer 5 PAL to the destination sol staker stake.
    let mut sol_staker_move_tokens = SolStakerMoveTokens {
        config: config_manager.config,
        vault_holder_rewards: config_manager.vault_holder_rewards,
        sol_staker_authority: source_sol_staker_staker_manager.authority.pubkey(),
        source_sol_staker_stake: source_sol_staker_staker_manager.stake,
        destination_sol_staker_stake: destination_sol_staker_staker_manager.stake,
    }
    .instruction(SolStakerMoveTokensInstructionArgs { amount: 5 });
    sol_staker_move_tokens.accounts[2].is_signer = false;
    let tx = Transaction::new_signed_with_payer(
        &[sol_staker_move_tokens],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Assert.
    assert_instruction_error!(err, InstructionError::MissingRequiredSignature);
}

#[tokio::test]
async fn transfer_with_wrong_authority_err() {
    let mut context = setup::setup(&[]).await;

    // Setup the relevant accounts.
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let stake_authority = Keypair::new();
    let source_sol_staker_staker_manager = SolStakerStakeManager::new_with_authority(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        stake_authority.insecure_clone(),
        5_000_000_000, // 5 SOL staked
    )
    .await;
    let destination_sol_staker_staker_manager = SolStakerStakeManager::new_with_authority(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        stake_authority,
        5_000_000_000, // 5 SOL staked
    )
    .await;

    // Stake 10 PAL on the source account.
    let mut source = get_account!(context, source_sol_staker_staker_manager.stake);
    let mut source_state = SolStakerStake::from_bytes(&source.data).unwrap();
    source_state.delegation.staked_amount = 10;
    source.data = source_state.try_to_vec().unwrap();
    context.set_account(&source_sol_staker_staker_manager.stake, &source.into());

    // Stake 10 PAL on the destination account.
    let mut destination = get_account!(context, destination_sol_staker_staker_manager.stake);
    let mut destination_state = SolStakerStake::from_bytes(&destination.data).unwrap();
    destination_state.delegation.staked_amount = 10;
    destination.data = destination_state.try_to_vec().unwrap();
    context.set_account(
        &destination_sol_staker_staker_manager.stake,
        &destination.into(),
    );

    // Act - Transfer 5 PAL to the destination sol staker stake.
    let wrong_authority = Keypair::new();
    let sol_staker_move_tokens = SolStakerMoveTokens {
        config: config_manager.config,
        vault_holder_rewards: config_manager.vault_holder_rewards,
        sol_staker_authority: wrong_authority.pubkey(),
        source_sol_staker_stake: source_sol_staker_staker_manager.stake,
        destination_sol_staker_stake: destination_sol_staker_staker_manager.stake,
    }
    .instruction(SolStakerMoveTokensInstructionArgs { amount: 5 });
    let tx = Transaction::new_signed_with_payer(
        &[sol_staker_move_tokens],
        Some(&context.payer.pubkey()),
        &[&context.payer, &wrong_authority],
        context.last_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Assert.
    assert_custom_error!(err, PaladinStakeProgramError::InvalidAuthority);
}

#[tokio::test]
async fn transfer_to_account_with_different_authority_err() {
    let mut context = setup::setup(&[]).await;

    // Setup the relevant accounts.
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let stake_authority = Keypair::new();
    let source_sol_staker_staker_manager = SolStakerStakeManager::new_with_authority(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        stake_authority.insecure_clone(),
        5_000_000_000, // 5 SOL staked
    )
    .await;
    let destination_sol_staker_staker_manager = SolStakerStakeManager::new(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        5_000_000_000, // 5 SOL staked
    )
    .await;

    // Stake 10 PAL on the source account.
    let mut source = get_account!(context, source_sol_staker_staker_manager.stake);
    let mut source_state = SolStakerStake::from_bytes(&source.data).unwrap();
    source_state.delegation.staked_amount = 10;
    source.data = source_state.try_to_vec().unwrap();
    context.set_account(&source_sol_staker_staker_manager.stake, &source.into());

    // Stake 10 PAL on the destination account.
    let mut destination = get_account!(context, destination_sol_staker_staker_manager.stake);
    let mut destination_state = SolStakerStake::from_bytes(&destination.data).unwrap();
    destination_state.delegation.staked_amount = 10;
    destination.data = destination_state.try_to_vec().unwrap();
    context.set_account(
        &destination_sol_staker_staker_manager.stake,
        &destination.into(),
    );

    // Act - Transfer 5 PAL to the destination sol staker stake.
    let wrong_authority = Keypair::new();
    let sol_staker_move_tokens = SolStakerMoveTokens {
        config: config_manager.config,
        vault_holder_rewards: config_manager.vault_holder_rewards,
        sol_staker_authority: wrong_authority.pubkey(),
        source_sol_staker_stake: source_sol_staker_staker_manager.stake,
        destination_sol_staker_stake: destination_sol_staker_staker_manager.stake,
    }
    .instruction(SolStakerMoveTokensInstructionArgs { amount: 5 });
    let tx = Transaction::new_signed_with_payer(
        &[sol_staker_move_tokens],
        Some(&context.payer.pubkey()),
        &[&context.payer, &wrong_authority],
        context.last_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Assert.
    assert_custom_error!(err, PaladinStakeProgramError::InvalidAuthority);
}
