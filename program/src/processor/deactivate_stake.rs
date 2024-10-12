use std::num::NonZeroU64;

use solana_program::{
    clock::Clock, entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey,
    sysvar::Sysvar,
};

use crate::{
    error::StakeError,
    instruction::accounts::{Context, DeactivateStakeAccounts},
    processor::unpack_delegation_mut,
    require,
    state::{Config, MAX_BASIS_POINTS},
};

/// Helper to calculate the maximum amount that can be deactivated.
#[inline(always)]
fn get_max_deactivation_amount(amount: u64, basis_points: u16) -> Result<u64, ProgramError> {
    let amount = (amount as u128)
        .checked_mul(basis_points as u128)
        .and_then(|p| p.checked_div(MAX_BASIS_POINTS))
        .ok_or(ProgramError::ArithmeticOverflow)?;
    Ok(amount as u64)
}

/// Deactivate staked tokens for a stake delegation.
///
/// Only one deactivation may be in-flight at once, so if this is called
/// with an active deactivation, it will succeed, but reset the amount and
/// timestamp.
///
/// 0. `[ ]` Stake config account
/// 1. `[w]` Validator or SOL staker stake account
/// 2. `[s]` Authority on validator stake account
///
/// Instruction data: amount of tokens to deactivate, as a little-endian `u64`.
pub fn process_deactivate_stake(
    program_id: &Pubkey,
    ctx: Context<DeactivateStakeAccounts>,
    amount: u64,
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

    let stake_data = &mut ctx.accounts.stake.try_borrow_mut_data()?;
    // checks that the stake account is initialized and has the correct derivation
    let delegation = unpack_delegation_mut(
        stake_data,
        ctx.accounts.stake.key,
        ctx.accounts.config.key,
        program_id,
    )?;

    // authority
    // - must be a signer
    // - must match the authority on the stake account

    require!(
        ctx.accounts.stake_authority.is_signer,
        ProgramError::MissingRequiredSignature,
        "stake_authority"
    );

    require!(
        ctx.accounts.stake_authority.key == &delegation.authority,
        StakeError::InvalidAuthority,
        "stake_authority"
    );

    // Validate the amount.
    require!(
        delegation.amount >= amount,
        StakeError::InsufficientStakeAmount
    );

    let max_deactivation_amount =
        get_max_deactivation_amount(delegation.amount, config.max_deactivation_basis_points)?;
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
    if amount > 0 {
        delegation.deactivating_amount = amount;
        delegation.deactivation_timestamp = NonZeroU64::new(Clock::get()?.unix_timestamp as u64);
    } else {
        // cancels the deactivation if the amount is 0
        delegation.deactivating_amount = 0;
        delegation.deactivation_timestamp = None;
    }

    Ok(())
}
