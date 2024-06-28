use {
    crate::instruction::accounts::{Context, HarvestHolderRewardsAccounts},
    solana_program::{entrypoint::ProgramResult, pubkey::Pubkey},
};

/// Harvests holder SOL rewards earned by the given stake account.
///
/// Rewards are added to the vault token account.
///
/// NOTE: This mostly replicates the logic in the rewards program. Since the
/// staked tokens are all held by this program, stakers need a way to access
/// their portion of holder rewards.
///
/// This instruction requires that `unclaimed_rewards` be equal to `0` in
/// the token vault account. For ease of use, be sure to call the
/// `HarvestRewards` on the vault account before this.
///
/// 0. `[]` Config account
/// 1. `[w]` Stake account
/// 2. `[w]` Vault token account
/// 3. `[]` Holder rewards account for vault token account
/// 4. `[w]` Destination account for withdrawn lamports
/// 5. `[s]` Stake authority
/// 6. `[]` Vault authority, PDA with seeds `['token-owner', stake_config]`
/// 7. `[]` Stake token mint, to get total supply
/// 8. `[]` SPL Token program
pub fn process_harvest_holder_rewards(
    _program_id: &Pubkey,
    _ctx: Context<HarvestHolderRewardsAccounts>,
) -> ProgramResult {
    Ok(())
}
