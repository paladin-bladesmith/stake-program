use solana_program::{
    clock::Clock, entrypoint::ProgramResult, msg, program_error::ProgramError, pubkey::Pubkey,
    sysvar::Sysvar,
};

use crate::{
    error::StakeError,
    instruction::accounts::{Context, InactivateValidatorStakeAccounts},
    processor::unpack_initialized_mut,
    require,
    state::{find_validator_stake_pda, Config, ValidatorStake},
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

    let data = &mut ctx.accounts.config.try_borrow_mut_data()?;
    let config = bytemuck::try_from_bytes_mut::<Config>(data)
        .map_err(|_error| ProgramError::InvalidAccountData)?;

    require!(
        config.is_initialized(),
        ProgramError::UninitializedAccount,
        "config",
    );

    // stake
    // - owner must be the stake program
    // - must be a ValidatorStake account
    // - must be initialized
    // - must have the correct derivation

    require!(
        ctx.accounts.stake.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "stake"
    );

    let stake_data = &mut ctx.accounts.stake.try_borrow_mut_data()?;
    let validator_stake = unpack_initialized_mut::<ValidatorStake>(stake_data)?;

    let (derivation, _) = find_validator_stake_pda(
        &validator_stake.delegation.validator_vote,
        ctx.accounts.config.key,
        program_id,
    );

    require!(
        ctx.accounts.stake.key == &derivation,
        ProgramError::InvalidSeeds,
        "stake",
    );

    let delegation = &mut validator_stake.delegation;

    // Inactivates the stake if elegible.

    if let Some(timestamp) = delegation.deactivation_timestamp {
        let inactive_timestamp = config.cooldown_time_seconds.saturating_add(timestamp.get());
        let current_timestamp = Clock::get()?.unix_timestamp as u64;

        require!(
            current_timestamp >= inactive_timestamp,
            StakeError::ActiveDeactivationCooldown,
            "{} second(s) remaining for deactivation",
            inactive_timestamp.saturating_sub(current_timestamp),
        );

        msg!("Inactivating {} token(s)", delegation.deactivating_amount);

        // moves deactivating amount to inactive
        delegation.amount = delegation
            .amount
            .checked_sub(delegation.deactivating_amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        delegation.inactive_amount = delegation
            .inactive_amount
            .checked_add(delegation.deactivating_amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        // Update the config and stake account data.

        config.token_amount_delegated = config
            .token_amount_delegated
            .checked_sub(delegation.deactivating_amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        // Clears the deactivation.

        delegation.deactivating_amount = 0;
        delegation.deactivation_timestamp = None;

        Ok(())
    } else {
        Err(StakeError::NoDeactivatedTokens.into())
    }
}
