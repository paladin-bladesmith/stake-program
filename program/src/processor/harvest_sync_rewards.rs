use paladin_sol_stake_view_program_client::{
    instructions::GetStakeActivatingAndDeactivatingCpiBuilder,
    GetStakeActivatingAndDeactivatingReturnData,
};
use solana_program::{
    entrypoint::ProgramResult, msg, program::get_return_data, program_error::ProgramError,
    pubkey::Pubkey,
};
use std::cmp::min;

use crate::{
    error::StakeError,
    instruction::accounts::{Context, HarvestSyncRewardsAccounts},
    processor::unpack_initialized_mut,
    require,
    state::{
        calculate_eligible_rewards, calculate_maximum_stake_for_lamports_amount,
        calculate_stake_rewards_per_token, find_sol_staker_stake_pda, find_validator_stake_pda,
        Config, SolStakerStake, ValidatorStake, EMPTY_RETURN_DATA,
    },
};

/// Harvest the rewards from syncing the SOL stake balance with a validator and SOL staker
/// stake accounts.
///
/// NOTE: Anybody can sync the balance of a SOL stake account.
///
/// 0. `[w]` Stake config account
/// 1. `[w]` SOL staker stake account
///     * PDA seeds: ['stake::state::sol_staker_stake', SOL stake, config]
/// 2. `[w]` Validator stake account
///     * PDA seeds: ['stake::state::validator_stake', validator, config]
/// 3. `[]` SOL stake account
/// 4. `[w]` Destination account
/// 5. `[]` Stake history sysvar
/// 6. `[]` Paladin SOL Stake View program
#[allow(clippy::useless_conversion)]
pub fn process_harvest_sync_rewards(
    program_id: &Pubkey,
    ctx: Context<HarvestSyncRewardsAccounts>,
) -> ProgramResult {
    // Accounts validation.

    // config
    // - owner must the stake program
    // - must be initialized

    require!(
        ctx.accounts.config.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "config"
    );

    let mut data = ctx.accounts.config.try_borrow_mut_data()?;
    let config = unpack_initialized_mut::<Config>(&mut data)?;

    // sol staker stake
    // - owner must be the stake program
    // - must be initialized
    // - must have the correct derivation (validates the config account)

    require!(
        ctx.accounts.sol_staker_stake.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "sol staker stake"
    );

    let mut sol_staker_stake_data = ctx.accounts.sol_staker_stake.try_borrow_mut_data()?;
    let sol_staker_stake = unpack_initialized_mut::<SolStakerStake>(&mut sol_staker_stake_data)?;

    let (derivation, _) = find_sol_staker_stake_pda(
        &sol_staker_stake.sol_stake,
        ctx.accounts.config.key,
        program_id,
    );

    require!(
        ctx.accounts.sol_staker_stake.key == &derivation,
        ProgramError::InvalidSeeds,
        "sol staker stake",
    );

    // validator stake
    // - owner must be the stake program
    // - must have the correct derivation (validates both the validator vote
    //   and config accounts)
    // - must be initialized

    require!(
        ctx.accounts.validator_stake.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "validator stake"
    );

    // validator vote must match the SOL staker stake state account's validator vote
    // (validation done on the derivation of the expected address)
    let (derivation, _) = find_validator_stake_pda(
        &sol_staker_stake.delegation.validator_vote,
        ctx.accounts.config.key,
        program_id,
    );

    require!(
        ctx.accounts.validator_stake.key == &derivation,
        ProgramError::InvalidSeeds,
        "validator stake",
    );

    let mut stake_data = ctx.accounts.validator_stake.try_borrow_mut_data()?;
    let validator_stake = unpack_initialized_mut::<ValidatorStake>(&mut stake_data)?;

    // stake state (validated by the SOL Stake View program)
    // - must match the one on the SOL staker stake account

    require!(
        ctx.accounts.sol_stake.key == &sol_staker_stake.sol_stake,
        StakeError::IncorrectSolStakeAccount,
        "sol stake"
    );

    require!(
        ctx.accounts.sol_stake_view_program.key == &paladin_sol_stake_view_program_client::ID,
        ProgramError::IncorrectProgramId,
        "invalid sol stake view program"
    );

    GetStakeActivatingAndDeactivatingCpiBuilder::new(ctx.accounts.sol_stake_view_program)
        .stake(ctx.accounts.sol_stake)
        .stake_history(ctx.accounts.sysvar_stake_history)
        .invoke()?;

    let (_, return_data) = get_return_data().ok_or(ProgramError::InvalidAccountData)?;
    let mut stake_state_data =
        bytemuck::try_from_bytes::<GetStakeActivatingAndDeactivatingReturnData>(&return_data)
            .map_err(|_error| ProgramError::InvalidAccountData)?;

    let delegated_vote = stake_state_data.delegated_vote.get();
    // If we have the correct sol stake account, but the delegated vote is not the same
    // as the validator vote, then the stake is not delegated to the SOL staker stake
    // account's validator vote anymore; we will clear the lamports amount in this case.
    if delegated_vote.is_none()
        || delegated_vote != Some(sol_staker_stake.delegation.validator_vote)
    {
        stake_state_data = bytemuck::from_bytes(&EMPTY_RETURN_DATA);
    }

    // Check whether the SOL stake is ouf-of-sync.

    let stake_amount = u64::from(stake_state_data.effective)
        .checked_add(u64::from(stake_state_data.activating))
        .ok_or(ProgramError::ArithmeticOverflow)?;

    if stake_amount == sol_staker_stake.lamports_amount {
        msg!("SOL stake is out-of-sync sync");

        // When the SOL stake is out-of-sync, we need to:
        //
        //   1. Update the SOL staker and validator stake accounts' lamports amount
        //
        //   2. Calculate the rewards earned by the SOL staker stake account, so we
        //      can determine how much fees the searcher should receive. Searcher fee
        //      is the min of (rewards available, config search fee)
        //
        //   3. Update the `last_seen_stake_rewards_per_token` on the SOL staker stake
        //      account to deduct the searcher fee. This will decrease the rewards per
        //      token for the current SOL staker stake account.
        //
        //   4. Transfer the searcher fee to the destination account

        // update lamports amount
        validator_stake.total_staked_lamports_amount = validator_stake
            .total_staked_lamports_amount
            .checked_sub(sol_staker_stake.lamports_amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        sol_staker_stake.lamports_amount = u64::from(stake_state_data.activating)
            .checked_add(stake_state_data.effective.into())
            .and_then(|amount| amount.checked_sub(u64::from(stake_state_data.deactivating)))
            .ok_or(ProgramError::ArithmeticOverflow)?;

        validator_stake.total_staked_lamports_amount = validator_stake
            .total_staked_lamports_amount
            .checked_add(sol_staker_stake.lamports_amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        // searcher rewards
        let stake_limit =
            calculate_maximum_stake_for_lamports_amount(sol_staker_stake.lamports_amount)?;
        let validated_amount = min(sol_staker_stake.delegation.amount, stake_limit as u64);

        let accumulated_rewards_per_token = u128::from(config.accumulated_stake_rewards_per_token);
        let rewards = calculate_eligible_rewards(
            accumulated_rewards_per_token,
            u128::from(
                sol_staker_stake
                    .delegation
                    .last_seen_stake_rewards_per_token,
            ),
            validated_amount,
        )?;
        let searcher_rewards = min(config.sync_rewards_lamports, rewards);

        // update last seen stake rewards per token
        let remaining_rewards = rewards
            .checked_sub(searcher_rewards)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        sol_staker_stake
            .delegation
            .last_seen_stake_rewards_per_token = calculate_stake_rewards_per_token(
            remaining_rewards,
            sol_staker_stake.delegation.amount,
        )?
        .into();

        // transfer searcher rewards
        let updated_config_lamports = ctx
            .accounts
            .config
            .lamports()
            .checked_sub(searcher_rewards)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        let updated_destination_lamports = ctx
            .accounts
            .destination
            .lamports()
            .checked_add(searcher_rewards)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        **ctx.accounts.config.try_borrow_mut_lamports()? = updated_config_lamports;
        **ctx.accounts.destination.try_borrow_mut_lamports()? = updated_destination_lamports;
    } else {
        msg!("SOL stake is in-sync");
    }

    Ok(())
}
