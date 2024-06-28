use {
    crate::instruction::accounts::{Context, HarvestStakeRewardsAccounts},
    solana_program::{entrypoint::ProgramResult, pubkey::Pubkey},
};

/// Harvests stake SOL rewards earned by the given stake account.
///
/// NOTE: This is very similar to the logic in the rewards program. Since the
/// staking rewards are held in a separate account, they must be distributed
/// based on the proportion of total stake.
///
/// 0. `[w]` Config account
/// 1. `[w]` Stake account
/// 2. `[w]` Destination account
/// 3. `[s]` Stake authority
pub fn process_harvest_stake_rewards(
    _program_id: &Pubkey,
    _ctx: Context<HarvestStakeRewardsAccounts>,
) -> ProgramResult {
    Ok(())
}
