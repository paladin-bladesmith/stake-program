use {
    crate::instruction::accounts::{Context, DistributeRewardsAccounts},
    solana_program::{entrypoint::ProgramResult, pubkey::Pubkey},
};

/// Moves SOL rewards to the config and updates the stake rewards total
///
/// Accounts expected by this instruction:
///
/// 0. `[w,s]` Reward payer
/// 1. `[w]` Config account
/// 2. `[]` System Program
pub fn process_distribute_rewards(
    _program_id: &Pubkey,
    _ctx: Context<DistributeRewardsAccounts>,
    _amount: u64,
) -> ProgramResult {
    Ok(())
}
