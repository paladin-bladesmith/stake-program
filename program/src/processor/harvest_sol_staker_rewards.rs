use paladin_sol_stake_view_program_client::{
    instructions::GetStakeActivatingAndDeactivatingCpiBuilder,
    GetStakeActivatingAndDeactivatingReturnData,
};
use solana_program::{
    entrypoint::ProgramResult, program::get_return_data, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    error::StakeError,
    instruction::accounts::{Context, HarvestSolStakerRewardsAccounts},
    processor::{harvest, unpack_initialized, unpack_initialized_mut},
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
/// 1. `[w]` Staker PAL stake account
/// 2. `[w]` Staker authority
/// 3. `[ ]` Staker native stake account
/// 4. `[w]` Validator stake account
/// 5. `[w]` Validator stake authority
/// 6. `[ ]` Stake history sysvar
/// 7. `[ ]` Paladin SOL stake view program
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

    let config_data = ctx.accounts.config.try_borrow_data()?;
    let config = unpack_initialized::<Config>(&config_data)?;

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
    let sol_staker_stake = unpack_initialized_mut::<SolStakerStake>(&mut stake_data)?;

    let (derivation, _) = find_sol_staker_stake_pda(
        &sol_staker_stake.sol_stake,
        ctx.accounts.config.key,
        program_id,
    );

    require!(
        ctx.accounts.sol_staker_stake.key == &derivation,
        ProgramError::InvalidSeeds,
        "sol stake stake",
    );

    // stake authority
    // - must match the authority on the stake account
    require!(
        ctx.accounts.stake_authority.key == &sol_staker_stake.delegation.authority,
        StakeError::InvalidAuthority,
        "stake authority",
    );

    // Native stake.
    // - Must match the PAL staker specified stake account.
    require!(
        ctx.accounts.native_stake.key == &sol_staker_stake.sol_stake,
        StakeError::IncorrectSolStakeAccount,
        "sol stake"
    );

    // Sol stake view program.
    // - Must match the expected program ID.
    require!(
        ctx.accounts.sol_stake_view_program.key == &paladin_sol_stake_view_program_client::ID,
        ProgramError::IncorrectProgramId,
        "invalid sol stake view program"
    );

    // Compute the latest native stake for this staker.
    GetStakeActivatingAndDeactivatingCpiBuilder::new(ctx.accounts.sol_stake_view_program)
        .stake(ctx.accounts.native_stake)
        .stake_history(ctx.accounts.sysvar_stake_history)
        .invoke()?;
    let (_, return_data) = get_return_data().ok_or(ProgramError::InvalidAccountData)?;
    let stake_state_data =
        bytemuck::try_from_bytes::<GetStakeActivatingAndDeactivatingReturnData>(&return_data)
            .map_err(|_error| ProgramError::InvalidAccountData)?;
    let delegated_vote = stake_state_data.delegated_vote.get();
    let delegate_changed = delegated_vote != Some(sol_staker_stake.delegation.validator_vote);
    let stake_amount = match delegate_changed {
        // TODO: We zero their effective PAL, but how do they re-activate it?
        true => panic!("Delegation changed"),
        false => u64::from(stake_state_data.activating)
            .checked_add(stake_state_data.effective.into())
            .and_then(|amount| amount.checked_sub(u64::from(stake_state_data.deactivating)))
            .ok_or(ProgramError::ArithmeticOverflow)?,
    };

    // If there is a difference, perform necessary steps to sync the accounts (and pay a sync bounty).
    if stake_amount != sol_staker_stake.lamports_amount {
        todo!(
            "Sync native stake; left={stake_amount}; right={}",
            sol_staker_stake.lamports_amount
        );
    }

    // Process the harvest.
    harvest(
        (config, ctx.accounts.config),
        &mut sol_staker_stake.delegation,
        ctx.accounts.stake_authority,
        None,
    )?;

    Ok(())
}
