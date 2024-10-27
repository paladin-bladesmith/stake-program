#![cfg(feature = "test-sbf")]

use borsh::BorshSerialize;
use paladin_stake_program_client::accounts::Config;
use paladin_stake_program_client::accounts::SolStakerStake;
use paladin_stake_program_client::errors::PaladinStakeProgramError;
use paladin_stake_program_client::instructions::{
    SolStakerUpdateAuthority, SolStakerUpdateAuthorityInstructionArgs,
};
use setup::validator_stake::ValidatorStakeManager;
use setup::{config::ConfigManager, sol_staker_stake::SolStakerStakeManager};
use solana_program_test::tokio;
use solana_sdk::instruction::InstructionError;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;

mod setup;

#[tokio::test]
async fn update_authority_zero_stake() {
    let mut context = setup::setup(&[]).await;

    // Setup the relevant accounts.
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let stake_authority = Keypair::new();
    let sol_staker_staker_manager = SolStakerStakeManager::new_with_authority(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        stake_authority.insecure_clone(),
        5_000_000_000, // 5 SOL staked
    )
    .await;

    let new_authority = Keypair::new();

    // Act - Update the authority.
    let sol_staker_update_authority = SolStakerUpdateAuthority {
        config: config_manager.config,
        config_authority: config_manager.authority.pubkey(),
        sol_staker_stake: sol_staker_staker_manager.stake,
    }
    .instruction(SolStakerUpdateAuthorityInstructionArgs {
        new_authority: new_authority.pubkey(),
    });
    let tx = Transaction::new_signed_with_payer(
        &[sol_staker_update_authority],
        Some(&context.payer.pubkey()),
        &[&context.payer, &config_manager.authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Assert - Authority has been changed.
    let stake = get_account!(context, sol_staker_staker_manager.stake);
    let stake = SolStakerStake::from_bytes(&stake.data).unwrap();
    assert_eq!(stake.delegation.authority, new_authority.pubkey());
}

#[tokio::test]
async fn update_authority_non_zero_stake() {
    let mut context = setup::setup(&[]).await;

    // Setup the relevant accounts.
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let stake_authority = Keypair::new();
    let sol_staker_staker_manager = SolStakerStakeManager::new_with_authority(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        stake_authority.insecure_clone(),
        5_000_000_000, // 5 SOL staked
    )
    .await;

    // Stake 10 PAL on the stake account.
    let mut stake = get_account!(context, sol_staker_staker_manager.stake);
    let mut stake_state = SolStakerStake::from_bytes(&stake.data).unwrap();
    stake_state.delegation.active_amount = 10;
    stake.data = stake_state.try_to_vec().unwrap();
    context.set_account(&sol_staker_staker_manager.stake, &stake.into());

    // Act - Update the authority.
    let new_authority = Keypair::new();
    let sol_staker_update_authority = SolStakerUpdateAuthority {
        config: config_manager.config,
        config_authority: config_manager.authority.pubkey(),
        sol_staker_stake: sol_staker_staker_manager.stake,
    }
    .instruction(SolStakerUpdateAuthorityInstructionArgs {
        new_authority: new_authority.pubkey(),
    });
    let tx = Transaction::new_signed_with_payer(
        &[sol_staker_update_authority],
        Some(&context.payer.pubkey()),
        &[&context.payer, &config_manager.authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Assert - Authority has been changed & stake amount is unaffected.
    let stake = get_account!(context, sol_staker_staker_manager.stake);
    let stake = SolStakerStake::from_bytes(&stake.data).unwrap();
    assert_eq!(stake.delegation.authority, new_authority.pubkey());
    assert_eq!(stake.delegation.active_amount, 10);
    assert_eq!(stake.delegation.effective_amount, 0);
    assert_eq!(stake.delegation.deactivating_amount, 0);
    assert_eq!(stake.delegation.inactive_amount, 0);
}

#[tokio::test]
async fn update_authority_config_authority_not_set_err() {
    let mut context = setup::setup(&[]).await;

    // Setup the relevant accounts.
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let stake_authority = Keypair::new();
    let sol_staker_staker_manager = SolStakerStakeManager::new_with_authority(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        stake_authority.insecure_clone(),
        5_000_000_000, // 5 SOL staked
    )
    .await;

    // Remove the config authority.
    let mut config = get_account!(context, config_manager.config);
    let mut config_state = Config::from_bytes(&config.data).unwrap();
    config_state.authority = Pubkey::default().into();
    config.data = config_state.try_to_vec().unwrap();
    context.set_account(&config_manager.config, &config.into());

    // Act - Update the authority.
    let new_authority = Keypair::new();
    let sol_staker_update_authority = SolStakerUpdateAuthority {
        config: config_manager.config,
        config_authority: config_manager.authority.pubkey(),
        sol_staker_stake: sol_staker_staker_manager.stake,
    }
    .instruction(SolStakerUpdateAuthorityInstructionArgs {
        new_authority: new_authority.pubkey(),
    });
    let tx = Transaction::new_signed_with_payer(
        &[sol_staker_update_authority],
        Some(&context.payer.pubkey()),
        &[&context.payer, &config_manager.authority],
        context.last_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Assert - Transaction errored due to no configured authority.
    assert_custom_error!(err, PaladinStakeProgramError::AuthorityNotSet);
}

#[tokio::test]
async fn update_authority_config_authority_signature_err() {
    let mut context = setup::setup(&[]).await;

    // Setup the relevant accounts.
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let stake_authority = Keypair::new();
    let sol_staker_staker_manager = SolStakerStakeManager::new_with_authority(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        stake_authority.insecure_clone(),
        5_000_000_000, // 5 SOL staked
    )
    .await;

    // Act - Update the authority.
    let new_authority = Keypair::new();
    let mut sol_staker_update_authority = SolStakerUpdateAuthority {
        config: config_manager.config,
        config_authority: config_manager.authority.pubkey(),
        sol_staker_stake: sol_staker_staker_manager.stake,
    }
    .instruction(SolStakerUpdateAuthorityInstructionArgs {
        new_authority: new_authority.pubkey(),
    });
    sol_staker_update_authority.accounts[1].is_signer = false;
    let tx = Transaction::new_signed_with_payer(
        &[sol_staker_update_authority],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    // Assert - Transaction errored due to no configured authority.
    assert_instruction_error!(err, InstructionError::MissingRequiredSignature);
}
