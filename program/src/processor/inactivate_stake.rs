use solana_program::{entrypoint::ProgramResult, pubkey::Pubkey};

use crate::instruction::accounts::{Context, InactivateStakeAccounts};

/// Move tokens from deactivating to inactive.
///
/// Reduces the total voting power for the stake account and the total staked
/// amount on the system.
///
/// NOTE: This instruction is permissionless, so anybody can finish
/// deactivating someone's tokens, preparing them to be withdrawn.
///
/// 0. `[w]` Stake config account
/// 1. `[w]` Validator stake account
pub fn process_inactivate_stake(
    _program_id: &Pubkey,
    _ctx: Context<InactivateStakeAccounts>,
) -> ProgramResult {
    Ok(())
}
