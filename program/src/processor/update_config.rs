use solana_program::{entrypoint::ProgramResult, pubkey::Pubkey};

use crate::instruction::{
    accounts::{Context, UpdateConfigAccounts},
    ConfigField,
};

/// Updates configuration parameters.
///
/// ### Accounts:
///
///   0. `[w]` config
///   1. `[s]` config_authority
pub fn process_update_config(
    _program_id: &Pubkey,
    _ctx: Context<UpdateConfigAccounts>,
    _field: ConfigField,
) -> ProgramResult {
    Ok(())
}
