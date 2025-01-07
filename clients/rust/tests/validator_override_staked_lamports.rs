#![cfg(feature = "test-sbf")]

use paladin_stake_program_client::accounts::ValidatorStake;
use paladin_stake_program_client::instructions::SolStakerSetAuthorityOverride;
use paladin_stake_program_client::instructions::SolStakerSetAuthorityOverrideInstructionArgs;
use paladin_stake_program_client::instructions::ValidatorOverrideStakedLamports;
use paladin_stake_program_client::instructions::ValidatorOverrideStakedLamportsInstructionArgs;
use setup::validator_stake::ValidatorStakeManager;
use setup::{config::ConfigManager, sol_staker_stake::SolStakerStakeManager};
use solana_program_test::tokio;
use solana_sdk::instruction::InstructionError;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::system_program;
use solana_sdk::transaction::Transaction;

mod setup;

#[tokio::test]
async fn validator_override_staked_lamports_ok() {
    let mut context = setup::setup(&[]).await;

    // Setup the relevant accounts.
    let config = ConfigManager::new(&mut context).await;
    let validator_stake_manager = ValidatorStakeManager::new(&mut context, &config.config).await;
    let stake_authority = Keypair::new();
    let validator_stake_manager = ValidatorStakeManager::new(&mut context, &config.config).await;

    // Set the PAL amount to 5.
    let mut validator_stake = get_account!(context, validator_stake_manager.stake);
    let mut validator_stake_state = ValidatorStake::from_bytes(&validator_stake.data).unwrap();
    validator_stake_state.delegation.active_amount = 2 * 10u64.pow(9);
    assert_eq!(
        validator_stake_state.delegation.active_amount,
        2 * 10u64.pow(9)
    );
    assert_eq!(validator_stake_state.delegation.effective_amount, 0);
    validator_stake.data = borsh::to_vec(&validator_stake_state).unwrap();
    context.set_account(&validator_stake_manager.stake, &validator_stake.into());

    // Act - Update the authority.
    let authority_override = Pubkey::new_unique();
    let mut sol_staker_update_authority = ValidatorOverrideStakedLamports {
        config: config.config,
        config_authority: config.authority.pubkey(),
        validator_stake: validator_stake_manager.stake,
        validator_stake_authority: validator_stake_manager.authority.pubkey(),
        vault_holder_rewards: config.vault_holder_rewards,
    }
    .instruction(ValidatorOverrideStakedLamportsInstructionArgs {
        amount_min: 10 * 10u64.pow(9),
    });
    let tx = Transaction::new_signed_with_payer(
        &[sol_staker_update_authority],
        Some(&context.payer.pubkey()),
        &[&context.payer, &config.authority],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Assert - The effective stake of the validator is now based on 10 SOL.
    let validator_stake = get_account!(context, validator_stake_manager.stake);
    let validator_stake = ValidatorStake::from_bytes(&validator_stake.data).unwrap();
    assert_eq!(validator_stake.total_staked_lamports_amount, 0);
    assert_eq!(
        validator_stake.total_staked_lamports_amount_min,
        10 * 10u64.pow(9)
    );
    assert_eq!(validator_stake.delegation.active_amount, 2 * 10u64.pow(9));
    assert_eq!(
        validator_stake.delegation.effective_amount,
        2 * 10u64.pow(9)
    );
}
