use solana_program::{entrypoint::ProgramResult, pubkey::Pubkey};

use crate::instruction::{
    accounts::{Context, SetAuthorityAccounts},
    Authority,
};

/// Sets new authority on a config or stake account
///
/// 0. `[w]` Config or stake account
/// 1. `[s]` Current authority
/// 2. `[]` New authority
pub fn process_set_authority(
    _program_id: &Pubkey,
    _ctx: Context<SetAuthorityAccounts>,
    _authority: Authority,
) -> ProgramResult {
    Ok(())
}
