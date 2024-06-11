use solana_program::{entrypoint::ProgramResult, pubkey::Pubkey};

use crate::instruction::{
    accounts::{Context, UpdateConfigAccounts},
    ConfigField,
};

/// Updates configuration parameters
///
/// 0. `[w]` Config account
/// 1. `[s]` Config authority
pub fn process_update_config(
    _program_id: &Pubkey,
    _ctx: Context<UpdateConfigAccounts>,
    _field: ConfigField,
) -> ProgramResult {
    Ok(())
}
