#![cfg(feature = "test-sbf")]
use borsh::BorshSerialize;
use paladin_stake_program_client::accounts::SolStakerStake;
use paladin_stake_program_client::instructions::SolStakerUpdateAuthority;
use setup::validator_stake::ValidatorStakeManager;
use setup::{config::ConfigManager, sol_staker_stake::SolStakerStakeManager};
use solana_program_test::tokio;
use solana_sdk::account::Account;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;

mod setup;

#[tokio::test]
async fn update_authority_zero_stake() {
    let mut context = setup::setup(&[]).await;
    let rent = context.banks_client.get_rent().await.unwrap();

    // Setup the relevant accounts.
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let stake_authority = Keypair::new();
    let sol_staker_stake_manager = SolStakerStakeManager::new_with_authority(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        stake_authority.insecure_clone(),
        5_000_000_000, // 5 SOL staked
    )
    .await;

    // Set the authority override with the new authority.
    let (sol_staker_authority_override, _) =
        paladin_stake_program_client::pdas::find_sol_staker_authority_override_pda(
            &sol_staker_stake_manager.authority.pubkey(),
            &config_manager.config,
        );
    let new_authority = Pubkey::new_unique();
    context.set_account(
        &sol_staker_authority_override,
        &Account {
            lamports: rent.minimum_balance(32),
            data: new_authority.to_bytes().to_vec(),
            owner: paladin_stake_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    // Act - Update the authority.
    let sol_staker_update_authority = SolStakerUpdateAuthority {
        config: config_manager.config,
        sol_staker_stake: sol_staker_stake_manager.stake,
        sol_staker_authority_override,
    }
    .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[sol_staker_update_authority],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Assert - Authority has been changed.
    let stake = get_account!(context, sol_staker_stake_manager.stake);
    let stake = SolStakerStake::from_bytes(&stake.data).unwrap();
    assert_eq!(stake.delegation.authority, new_authority);
}

#[tokio::test]
async fn update_authority_non_zero_stake() {
    let mut context = setup::setup(&[]).await;
    let rent = context.banks_client.get_rent().await.unwrap();

    // Setup the relevant accounts.
    let config_manager = ConfigManager::new(&mut context).await;
    let validator_stake_manager =
        ValidatorStakeManager::new(&mut context, &config_manager.config).await;
    let stake_authority = Keypair::new();
    let sol_staker_stake_manager = SolStakerStakeManager::new_with_authority(
        &mut context,
        &config_manager.config,
        &validator_stake_manager.stake,
        &validator_stake_manager.vote,
        stake_authority.insecure_clone(),
        5_000_000_000, // 5 SOL staked
    )
    .await;

    // Stake 10 PAL on the stake account.
    let mut stake = get_account!(context, sol_staker_stake_manager.stake);
    let mut stake_state = SolStakerStake::from_bytes(&stake.data).unwrap();
    stake_state.delegation.staked_amount = 10;
    stake.data = stake_state.try_to_vec().unwrap();
    context.set_account(&sol_staker_stake_manager.stake, &stake.into());

    // Set the authority override with the new authority.
    let new_authority = Pubkey::new_unique();
    let (sol_staker_authority_override, _) =
        paladin_stake_program_client::pdas::find_sol_staker_authority_override_pda(
            &sol_staker_stake_manager.authority.pubkey(),
            &config_manager.config,
        );
    context.set_account(
        &sol_staker_authority_override,
        &Account {
            lamports: rent.minimum_balance(32),
            data: new_authority.to_bytes().to_vec(),
            owner: paladin_stake_program_client::ID,
            executable: false,
            rent_epoch: 0,
        }
        .into(),
    );

    // Act - Update the authority.
    let sol_staker_update_authority = SolStakerUpdateAuthority {
        config: config_manager.config,
        sol_staker_stake: sol_staker_stake_manager.stake,
        sol_staker_authority_override: sol_staker_authority_override,
    }
    .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[sol_staker_update_authority],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Assert - Authority has been changed & stake amount is unaffected.
    let stake = get_account!(context, sol_staker_stake_manager.stake);
    let stake = SolStakerStake::from_bytes(&stake.data).unwrap();
    assert_eq!(stake.delegation.authority, new_authority);
    assert_eq!(stake.delegation.staked_amount, 10);
    assert_eq!(stake.delegation.effective_amount, 0);
}
