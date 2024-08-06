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
    instruction::accounts::{Context, SyncSolStakeAccounts},
    processor::unpack_initialized_mut,
    require,
    state::{
        find_sol_staker_stake_pda, find_validator_stake_pda, Config, SolStakerStake, ValidatorStake,
    },
};

/// Sync the SOL stake balance with a validator and SOL staker stake accounts.
///
/// NOTE: Anybody can sync the balance of a SOL stake account.
///
/// 0. `[w]` SOL staker stake account
///     * PDA seeds: ['stake::state::sol_staker_stake', SOL stake, config]
/// 1. `[w]` Validator stake account
///     * PDA seeds: ['stake::state::validator_stake', validator, config]
/// 2. `[]` SOL stake account
/// 3. `[]` Stake history sysvar
/// 4. `[]` Paladin SOL Stake View program
#[allow(clippy::useless_conversion)]
pub fn process_sync_sol_stake(
    program_id: &Pubkey,
    ctx: Context<SyncSolStakeAccounts>,
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

    let data = &ctx.accounts.config.try_borrow_data()?;

    let config = bytemuck::try_from_bytes::<Config>(data)
        .map_err(|_error| ProgramError::InvalidAccountData)?;

    require!(
        config.is_initialized(),
        ProgramError::UninitializedAccount,
        "config"
    );

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
    let stake_state_data =
        bytemuck::try_from_bytes::<GetStakeActivatingAndDeactivatingReturnData>(&return_data)
            .map_err(|_error| ProgramError::InvalidAccountData)?;

    // Updates the SOL stake on both the validator and SOL staker stake accounts.
    //
    //   1. Substract the previous stake amount from the validator stake account.
    //
    //   2. Updates the new stake amount on the SOL staker stake account.
    //
    //   3. Add the new stake amount to the validator stake account.

    validator_stake.total_staked_lamports_amount = validator_stake
        .total_staked_lamports_amount
        .checked_sub(sol_staker_stake.lamports_amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    sol_staker_stake.lamports_amount = u64::from(stake_state_data.activating)
        .checked_add(stake_state_data.effective.into())
        .ok_or(ProgramError::ArithmeticOverflow)?;

    validator_stake.total_staked_lamports_amount = validator_stake
        .total_staked_lamports_amount
        .checked_add(sol_staker_stake.lamports_amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    Ok(())
}
