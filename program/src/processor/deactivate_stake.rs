use std::num::NonZeroU64;

use solana_program::{
    clock::Clock, entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey,
    sysvar::Sysvar,
};

use crate::{
    error::StakeError,
    instruction::accounts::{Context, DeactivateStakeAccounts},
    require,
    state::{find_stake_pda, Config, Stake},
};

/// Helper to calculate the maximum amount that can be deactivated.
#[inline(always)]
fn get_max_deactivation_amount(total_amount: u64, basis_points: u16) -> Result<u64, ProgramError> {
    total_amount
        .checked_mul(basis_points as u64)
        .ok_or(ProgramError::ArithmeticOverflow)?
        .checked_div(10_000)
        .ok_or(ProgramError::ArithmeticOverflow)
}

/// Deactivate staked tokens for the validator.
///
/// Only one deactivation may be in-flight at once, so if this is called
/// with an active deactivation, it will succeed, but reset the amount and
/// timestamp.
///
/// 0. `[]` Stake config account
/// 1. `[w]` Validator stake account
/// 2. `[s]` Authority on validator stake account
///
/// Instruction data: amount of tokens to deactivate, as a little-endian u64
pub fn process_deactivate_stake(
    program_id: &Pubkey,
    ctx: Context<DeactivateStakeAccounts>,
    amount: u64,
) -> ProgramResult {
    // Account valuidation.

    // config
    // - owner must be the stake program
    // - must be initialized

    require!(
        ctx.accounts.config.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "config"
    );

    let data = &ctx.accounts.config.try_borrow_data()?;
    let config = bytemuck::try_from_bytes::<Config>(data)
        .map_err(|_error| ProgramError::InvalidAccountData)?;

    require!(
        config.is_initialized(),
        ProgramError::UninitializedAccount,
        "config",
    );

    // stake
    // - owner must be the stake program
    // - must be initialized
    // - must have the correct derivation

    require!(
        ctx.accounts.stake.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "stake"
    );

    let data = &mut ctx.accounts.stake.try_borrow_mut_data()?;
    let stake = bytemuck::try_from_bytes_mut::<Stake>(data)
        .map_err(|_error| ProgramError::InvalidAccountData)?;

    require!(
        stake.is_initialized(),
        ProgramError::UninitializedAccount,
        "stake",
    );

    // validates that the stake account corresponds to the received config account
    let (derivation, _) = find_stake_pda(&stake.validator, ctx.accounts.config.key, program_id);

    require!(
        ctx.accounts.stake.key == &derivation,
        ProgramError::InvalidSeeds,
        "stake",
    );

    // authority
    // - must be a signer
    // - must match the authority on the stake account

    require!(
        ctx.accounts.stake_authority.is_signer,
        ProgramError::MissingRequiredSignature,
        "stake_authority"
    );

    require!(
        ctx.accounts.stake_authority.key == &stake.authority,
        StakeError::InvalidAuthority,
        "stake_authority"
    );

    // Validate the amount.

    require!(amount > 0, StakeError::InvalidAmount);

    require!(stake.amount >= amount, StakeError::InsufficientStakeAmount);

    let max_deactivation_amount = get_max_deactivation_amount(
        config.token_amount_delegated,
        config.max_deactivation_basis_points,
    )?;

    require!(
        amount <= max_deactivation_amount,
        StakeError::MaximumDeactivationAmountExceeded,
        "amount requested ({}), maximum allowed ({})",
        amount,
        max_deactivation_amount
    );

    // Deactivate the stake.
    //
    // If the stake is already deactivating, this will reset the deactivation.

    stake.deactivating_amount = amount;
    stake.deactivation_timestamp = NonZeroU64::new(Clock::get()?.unix_timestamp as u64);

    Ok(())
}
