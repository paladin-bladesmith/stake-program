use solana_program::{entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey};

use crate::{
    error::StakeError,
    instruction::accounts::{Context, HarvestSolStakerRewardsAccounts},
    processor::{process_harvest_for_delegation, unpack_initialized_mut},
    require,
    state::{find_sol_staker_stake_pda, Config, SolStakerStake},
};

/// Harvests stake SOL rewards earned by the given SOL staker stake account.
///
/// NOTE: This is very similar to the logic in the rewards program. Since the
/// staking rewards are held in a separate account, they must be distributed
/// based on the proportion of total stake.
///
/// 0. `[w]` Config account
/// 1. `[w]` SOL staker stake account
/// 2. `[w]` Destination account
/// 3. `[s]` Stake authority
pub fn process_harvest_sol_staker_rewards(
    program_id: &Pubkey,
    ctx: Context<HarvestSolStakerRewardsAccounts>,
) -> ProgramResult {
    // Account validation.

    // config
    // - owner must be the stake program
    // - must be initialized

    require!(
        ctx.accounts.config.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "config"
    );

    let mut config_data = ctx.accounts.config.try_borrow_mut_data()?;
    let config = unpack_initialized_mut::<Config>(&mut config_data)?;

    // sol staker stake
    // - owner must be the stake program
    // - must be initialized
    // - derivation must match (validates the config account)

    require!(
        ctx.accounts.sol_staker_stake.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "sol staker stake"
    );

    let mut stake_data = ctx.accounts.sol_staker_stake.try_borrow_mut_data()?;
    let sol_stake_stake = unpack_initialized_mut::<SolStakerStake>(&mut stake_data)?;

    let (derivation, _) = find_sol_staker_stake_pda(
        &sol_stake_stake.sol_stake,
        ctx.accounts.config.key,
        program_id,
    );

    require!(
        ctx.accounts.sol_staker_stake.key == &derivation,
        ProgramError::InvalidSeeds,
        "sol stake stake",
    );

    // stake authority
    // - must be a signer
    // - must match the authority on the stake account

    require!(
        ctx.accounts.stake_authority.is_signer,
        ProgramError::MissingRequiredSignature,
        "stake authority",
    );

    require!(
        ctx.accounts.stake_authority.key == &sol_stake_stake.delegation.authority,
        StakeError::InvalidAuthority,
        "stake authority",
    );

    // Process the harvest.

    process_harvest_for_delegation(
        config,
        &mut sol_stake_stake.delegation,
        ctx.accounts.config,
        ctx.accounts.destination,
    )
}
