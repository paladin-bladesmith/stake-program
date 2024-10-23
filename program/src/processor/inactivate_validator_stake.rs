use solana_program::{
    clock::Clock, entrypoint::ProgramResult, msg, program_error::ProgramError, pubkey::Pubkey,
    sysvar::Sysvar,
};

use crate::{
    error::StakeError,
    instruction::accounts::{Context, InactivateValidatorStakeAccounts},
    processor::{harvest, unpack_initialized_mut, HarvestAccounts},
    require,
    state::{
        calculate_maximum_stake_for_lamports_amount, find_validator_stake_pda, Config,
        ValidatorStake,
    },
};

/// Move tokens from deactivating to inactive.
///
/// Reduces the total voting power for the validator stake account and the total staked
/// amount on the system.
///
/// NOTE: This instruction is permissionless, so anybody can finish
/// deactivating validator's tokens, preparing them to be withdrawn.
///
/// 0. `[w]` Stake config account
/// 1. `[w]` Validator stake account
pub fn process_inactivate_validator_stake(
    program_id: &Pubkey,
    ctx: Context<InactivateValidatorStakeAccounts>,
) -> ProgramResult {
    // Account validation.

    // config
    // - owner must be the stake program
    // - must be initialized
    require!(
        ctx.accounts.config.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "config"
    );
    let mut config = ctx.accounts.config.data.borrow_mut();
    let config = unpack_initialized_mut::<Config>(&mut config)?;

    // stake
    // - owner must be the stake program
    // - must be a ValidatorStake account
    // - must be initialized
    // - must have the correct derivation
    require!(
        ctx.accounts.validator_stake.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "stake"
    );
    let stake_data = &mut ctx.accounts.validator_stake.try_borrow_mut_data()?;
    let stake = unpack_initialized_mut::<ValidatorStake>(stake_data)?;
    let (derivation, _) = find_validator_stake_pda(
        &stake.delegation.validator_vote,
        ctx.accounts.config.key,
        program_id,
    );
    require!(
        ctx.accounts.validator_stake.key == &derivation,
        ProgramError::InvalidSeeds,
        "stake",
    );
    let delegation = &mut stake.delegation;

    // Harvest rewards & update last claim tracking.
    harvest(
        HarvestAccounts {
            config: ctx.accounts.config,
            vault_holder_rewards: ctx.accounts.vault_holder_rewards,
            authority: ctx.accounts.validator_stake_authority,
        },
        config,
        delegation,
        None,
    )?;

    // Inactivates the stake if eligible.
    let Some(timestamp) = delegation.deactivation_timestamp else {
        return Err(StakeError::NoDeactivatedTokens.into());
    };
    let inactive_timestamp = config.cooldown_time_seconds.saturating_add(timestamp.get());
    let current_timestamp = Clock::get()?.unix_timestamp as u64;

    require!(
        current_timestamp >= inactive_timestamp,
        StakeError::ActiveDeactivationCooldown,
        "{} second(s) remaining for deactivation",
        inactive_timestamp.saturating_sub(current_timestamp),
    );

    msg!("Inactivating {} token(s)", delegation.deactivating_amount);

    // Compute the new stake values.
    let validator_active = delegation
        .active_amount
        .checked_sub(delegation.deactivating_amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    let validator_inactive = delegation
        .inactive_amount
        .checked_add(delegation.deactivating_amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    let validator_limit =
        calculate_maximum_stake_for_lamports_amount(stake.total_staked_lamports_amount)?;
    let validator_effective = std::cmp::min(validator_active, validator_limit);
    let effective_delta = delegation
        .effective_amount
        .checked_sub(validator_effective)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    // Update the state values.
    delegation.active_amount = validator_active;
    delegation.effective_amount = validator_effective;
    delegation.deactivating_amount = 0;
    delegation.deactivation_timestamp = None;
    delegation.inactive_amount = validator_inactive;
    config.token_amount_effective = config
        .token_amount_effective
        .checked_sub(effective_delta)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    Ok(())
}
