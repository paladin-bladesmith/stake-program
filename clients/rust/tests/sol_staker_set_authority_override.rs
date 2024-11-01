#![cfg(feature = "test-sbf")]

use borsh::BorshSerialize;
use paladin_stake_program_client::accounts::Config;
use paladin_stake_program_client::accounts::SolStakerStake;
use paladin_stake_program_client::errors::PaladinStakeProgramError;
use paladin_stake_program_client::instructions::SolStakerSetAuthorityOverride;
use paladin_stake_program_client::instructions::SolStakerSetAuthorityOverrideInstructionArgs;
use paladin_stake_program_client::instructions::SolStakerUpdateAuthority;
use setup::validator_stake::ValidatorStakeManager;
use setup::{config::ConfigManager, sol_staker_stake::SolStakerStakeManager};
use solana_program_test::tokio;
use solana_sdk::account::Account;
use solana_sdk::instruction::InstructionError;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::system_program;
use solana_sdk::transaction::Transaction;

mod setup;

#[tokio::test]
async fn config_authority_signature_err() {
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

    // Act - Update the authority.
    let authority_override = Pubkey::new_unique();
    let mut sol_staker_update_authority = SolStakerSetAuthorityOverride {
        config: config_manager.config,
        config_authority: config_manager.authority.pubkey(),
        sol_staker_authority_override: authority_override,
        system_program: Some(system_program::ID),
    }
    .instruction(SolStakerSetAuthorityOverrideInstructionArgs {
        authority_original: sol_staker_stake_manager.authority.pubkey(),
        authority_override: authority_override,
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
