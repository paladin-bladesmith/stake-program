use {
    crate::instruction::accounts::{Context, SlashAccounts},
    solana_program::{entrypoint::ProgramResult, pubkey::Pubkey},
};

/// Slashes a stake account for the given amount
///
/// Burns the given amount of tokens from the vault account, and reduces the
/// amount in the stake account.
///
/// 0. `[w]` Config account
/// 1. `[w]` Stake account
/// 2. `[s]` Slash authority
/// 3. `[w]` Vault token account
/// 4. `[]` Vault authority, PDA with seeds `['token-owner', stake_config]`
/// 5. `[]` SPL Token program
///
/// Instruction data: amount of tokens to slash
pub fn process_slash(
    _program_id: &Pubkey,
    _ctx: Context<SlashAccounts>,
    _amount: u64,
) -> ProgramResult {
    Ok(())
}
