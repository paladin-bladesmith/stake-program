use std::num::NonZeroU64;

use solana_program::{
    clock::Clock, entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey,
    sysvar::Sysvar,
};

use crate::{
    error::StakeError,
    instruction::accounts::{Context, DeactivateStakeAccounts},
    require,
    state::Stake,
};

/// Deactivate staked tokens for the validator.
///
/// Only one deactivation may be in-flight at once, so if this is called
/// with an active deactivation, it will succeed, but reset the amount and
/// timestamp.
///
/// 0. `[w]` Validator stake account
/// 1. `[s]` Authority on validator stake account
///
/// Instruction data: amount of tokens to deactivate, as a little-endian u64
pub fn process_deactivate_stake(
    program_id: &Pubkey,
    ctx: Context<DeactivateStakeAccounts>,
    amount: u64,
) -> ProgramResult {
    // Account valuidation.

    // stake
    // - owner must be the stake program
    // - must be initialized

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

    // Deactivate the stake.
    //
    // If the stake is already deactivating, this will reset the deactivation.

    stake.deactivating_amount = amount;
    stake.deactivation_timestamp = NonZeroU64::new(Clock::get()?.unix_timestamp as u64);

    Ok(())
}
