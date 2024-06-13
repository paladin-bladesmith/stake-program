use solana_program::{entrypoint::ProgramResult, pubkey::Pubkey};

use crate::instruction::accounts::{Context, InitializeConfigAccounts};

/// Creates Stake config account which controls staking parameters.
///
/// ### Accounts:
///
///   0. `[w]` config
///   1. `[]` authority
///   2. `[]` slash_authority
///   3. `[]` mint
///   4. `[]` vault_token
pub fn process_initialize_config(
    _program_id: &Pubkey,
    _ctx: Context<InitializeConfigAccounts>,
    _cooldown_time_seconds: u64,
    _max_deactivation_basis_points: u16,
) -> ProgramResult {
    Ok(())
}
