use solana_program::{entrypoint::ProgramResult, pubkey::Pubkey};

use crate::instruction::{
    accounts::{Context, SetAuthorityAccounts},
    AuthorityType,
};

/// Sets new authority on a config or stake account
///
/// ### Accounts:
///
///   0. `[w]` config
///   1. `[s]` config_authority
///   2. `[]` new_authority
pub fn process_set_authority(
    _program_id: &Pubkey,
    _ctx: Context<SetAuthorityAccounts>,
    _authority_type: AuthorityType,
) -> ProgramResult {
    Ok(())
}
