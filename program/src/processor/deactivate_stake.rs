use solana_program::{entrypoint::ProgramResult, pubkey::Pubkey};

use crate::instruction::accounts::{Context, DeactivateStakeAccounts};

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
    _program_id: &Pubkey,
    _ctx: Context<DeactivateStakeAccounts>,
    _amount: u64,
) -> ProgramResult {
    Ok(())
}
