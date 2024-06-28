use {
    crate::instruction::accounts::{Context, WithdrawInactiveStakeAccounts},
    solana_program::{entrypoint::ProgramResult, pubkey::Pubkey},
};

/// Withdraw inactive staked tokens from the vault
///
/// After a deactivation has gone through the cooldown period and been
/// "inactivated", the authority may move the tokens out of the vault.
///
/// 0. `[w]` Config account
/// 1. `[w]` Stake account
/// 2. `[w]` Vault token account
/// 3. `[w]` Destination token account
/// 4. `[s]` Stake authority
/// 5. `[]` Vault authority, PDA with seeds `['token-owner', stake_config]`
/// 6. `[]` SPL Token program
/// 7.. Extra required accounts for transfer hook
///
/// Instruction data: amount of tokens to move
pub fn process_withdraw_inactive_stake(
    _program_id: &Pubkey,
    _ctx: Context<WithdrawInactiveStakeAccounts>,
    _amount: u64,
) -> ProgramResult {
    Ok(())
}
