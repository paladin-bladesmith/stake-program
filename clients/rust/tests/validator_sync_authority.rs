#![cfg(feature = "test-sbf")]
use paladin_stake_program_client::accounts::ValidatorStake;
use paladin_stake_program_client::instructions::ValidatorSyncAuthority;
use setup::config::ConfigManager;
use setup::validator_stake::ValidatorStakeManager;
use solana_program_test::tokio;
use solana_sdk::instruction::InstructionError;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;

mod setup;

#[tokio::test]
async fn update_validator_authority_ok() {
    let mut context = setup::setup(&[]).await;

    // Setup the relevant accounts.
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    // Update the withdraw authority on the vote account.
    let new_authority = Pubkey::new_unique();
    let mut vote_account = get_account!(context, validator_stake_manager.vote);
    vote_account.data[36..68].copy_from_slice(&new_authority.to_bytes());
    context.set_account(&validator_stake_manager.vote, &vote_account.into());

    // Act - Update the authority.
    let validator_sync_authority = ValidatorSyncAuthority {
        config: config_manager.config,
        validator_stake: validator_stake_manager.stake,
        validator_vote: validator_stake_manager.vote,
    }
    .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[validator_sync_authority],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Assert - Authority has been changed.
    let stake = get_account!(context, validator_stake_manager.stake);
    let stake = ValidatorStake::from_bytes(&stake.data).unwrap();
    assert_eq!(stake.delegation.authority, new_authority);
}

#[tokio::test]
async fn update_validator_authority_err_invalid_config() {
    let mut context = setup::setup(&[]).await;

    // Setup the relevant accounts.
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    // Update the withdraw authority on the vote account.
    let new_authority = Pubkey::new_unique();
    let mut vote_account = get_account!(context, validator_stake_manager.vote);
    vote_account.data[36..68].copy_from_slice(&new_authority.to_bytes());
    context.set_account(&validator_stake_manager.vote, &vote_account.into());

    // Act - Update the authority.
    let validator_sync_authority = ValidatorSyncAuthority {
        config: Pubkey::new_unique(),
        validator_stake: validator_stake_manager.stake,
        validator_vote: validator_stake_manager.vote,
    }
    .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[validator_sync_authority],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Assert
    assert_instruction_error!(err, InstructionError::InvalidAccountOwner);
}

#[tokio::test]
async fn update_validator_authority_err_vote_mismatch() {
    let mut context = setup::setup(&[]).await;

    // Setup the relevant accounts.
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    // Update the withdraw authority on the vote account.
    let new_authority = Pubkey::new_unique();
    let mut vote_account = get_account!(context, validator_stake_manager.vote);
    vote_account.data[36..68].copy_from_slice(&new_authority.to_bytes());
    context.set_account(&validator_stake_manager.vote, &vote_account.into());

    // Create an additional vote account.
    let wrong_vote = ValidatorStakeManager::new(&mut context, &config_manager.config)
        .await
        .vote;

    // Act - Update the authority.
    let validator_sync_authority = ValidatorSyncAuthority {
        config: config_manager.config,
        validator_stake: validator_stake_manager.stake,
        validator_vote: wrong_vote,
    }
    .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[validator_sync_authority],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Assert
    assert_instruction_error!(err, InstructionError::InvalidSeeds);
}

#[tokio::test]
async fn update_validator_authority_err_stake_mismatch() {
    let mut context = setup::setup(&[]).await;

    // Setup the relevant accounts.
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;

    // Update the withdraw authority on the vote account.
    let new_authority = Pubkey::new_unique();
    let mut vote_account = get_account!(context, validator_stake_manager.vote);
    vote_account.data[36..68].copy_from_slice(&new_authority.to_bytes());
    context.set_account(&validator_stake_manager.vote, &vote_account.into());

    // Create an additional validator stake account.
    let wrong_validator_stake = ValidatorStakeManager::new(&mut context, &config_manager.config)
        .await
        .stake;

    // Act - Update the authority.
    let validator_sync_authority = ValidatorSyncAuthority {
        config: config_manager.config,
        validator_stake: wrong_validator_stake,
        validator_vote: validator_stake_manager.vote,
    }
    .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[validator_sync_authority],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Assert
    assert_instruction_error!(err, InstructionError::InvalidSeeds);
}
