use {
    crate::instruction::accounts::{Context, InitializeStakeAccounts},
    solana_program::{entrypoint::ProgramResult, pubkey::Pubkey},
};

/// Initializes stake account data for a validator.
///
/// NOTE: Anybody can create the stake account for a validator. For new
/// accounts, the authority is initialized to the validator vote account's
/// withdraw authority.
///
/// 0. `[]` Stake config account
/// 1. `[w]` Validator stake account
///     * PDA seeds: ['stake', validator, config_account]
/// 2. `[]` Validator vote account
/// 3. `[]` System program
pub fn process_initialize_stake(
    _program_id: &Pubkey,
    _ctx: Context<InitializeStakeAccounts>,
) -> ProgramResult {
    Ok(())
}
