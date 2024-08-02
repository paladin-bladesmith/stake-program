use paladin_sol_stake_view_program_client::{
    instructions::GetStakeActivatingAndDeactivatingCpiBuilder,
    GetStakeActivatingAndDeactivatingReturnData,
};
use solana_program::{
    entrypoint::ProgramResult,
    program::{get_return_data, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
};

use crate::{
    err,
    error::StakeError,
    instruction::accounts::{Context, InitializeSolStakerStakeAccounts},
    processor::unpack_initialized_mut,
    require,
    state::{
        find_sol_staker_stake_pda, find_validator_stake_pda, get_sol_staker_stake_pda_signer_seeds,
        Config, SolStakerStake, ValidatorStake,
    },
};

/// Initializes stake account data for a SOL staker.
///
/// There can be only one SOL staker stake account per stake state and config account, since
/// the stake state is part of the SOL staker stake account seeds.
///
/// NOTE: Anybody can create the stake account for a SOL staker. For new
/// accounts, the authority is initialized to the stake state account's withdrawer.
///
/// 0. `[]` Stake config account
/// 1. `[w]` SOL staker stake account
///     * PDA seeds: ['stake::state::sol_staker_stake', stake state, config]
/// 2. `[]` SOL stake state account
/// 3. `[]` Stake history sysvar
/// 4. `[]` System program
/// 5. `[]` Paladin SOL Stake View program
#[allow(clippy::useless_conversion)]
pub fn process_initialize_sol_staker_stake(
    program_id: &Pubkey,
    ctx: Context<InitializeSolStakerStakeAccounts>,
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

    // stake state (validated by the SOL Stake View program)
    // - must have a delegation

    GetStakeActivatingAndDeactivatingCpiBuilder::new(ctx.accounts.sol_stake_view_program)
        .stake(ctx.accounts.stake_state)
        .stake_history(ctx.accounts.stake_history)
        .invoke()?;

    let (_, return_data) = get_return_data().ok_or(ProgramError::InvalidAccountData)?;
    let stake_state_data =
        bytemuck::try_from_bytes::<GetStakeActivatingAndDeactivatingReturnData>(&return_data)
            .map_err(|_error| ProgramError::InvalidAccountData)?;

    let (withdrawer, validator_vote) =
        if let Some(delegated_vote) = stake_state_data.delegated_vote.get() {
            (
                // we should always have a withdrawer if the stake is delegated
                stake_state_data
                    .withdrawer
                    .get()
                    .ok_or(ProgramError::InvalidAccountData)?,
                delegated_vote,
            )
        } else {
            return err!(StakeError::UndelegatedStakeStateAccount);
        };

    // validator stake
    // - owner must be the stake program
    // - must have the correct derivation
    // - must be initialized

    require!(
        ctx.accounts.validator_stake.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "validator stake"
    );

    // validator vote must match the stake state account's validator vote (validation
    // done on the derivation of the expected address)
    let (derivation, _) =
        find_validator_stake_pda(&validator_vote, ctx.accounts.config.key, program_id);

    require!(
        ctx.accounts.validator_stake.key == &derivation,
        ProgramError::InvalidSeeds,
        "validator stake",
    );

    let mut stake_data = ctx.accounts.validator_stake.try_borrow_mut_data()?;
    let validator_stake = unpack_initialized_mut::<ValidatorStake>(&mut stake_data)?;

    // sol staker stake
    // - have the correct PDA derivation
    // - be uninitialized (empty data)
    //
    // NOTE: The stake account is created and assigned to the stake program, so it needs
    // to be pre-funded with the minimum rent balance by the caller.

    let (derivation, bump) = find_sol_staker_stake_pda(
        ctx.accounts.stake_state.key,
        ctx.accounts.config.key,
        program_id,
    );

    require!(
        ctx.accounts.sol_staker_stake.key == &derivation,
        ProgramError::InvalidSeeds,
        "stake"
    );

    require!(
        ctx.accounts.sol_staker_stake.data_is_empty(),
        ProgramError::AccountAlreadyInitialized,
        "stake"
    );

    // Allocate and assign.

    let bump_seed = [bump];
    let signer_seeds = get_sol_staker_stake_pda_signer_seeds(
        ctx.accounts.stake_state.key,
        ctx.accounts.config.key,
        &bump_seed,
    );

    invoke_signed(
        &system_instruction::allocate(
            ctx.accounts.sol_staker_stake.key,
            SolStakerStake::LEN as u64,
        ),
        &[ctx.accounts.sol_staker_stake.clone()],
        &[&signer_seeds],
    )?;

    invoke_signed(
        &system_instruction::assign(ctx.accounts.sol_staker_stake.key, program_id),
        &[ctx.accounts.sol_staker_stake.clone()],
        &[&signer_seeds],
    )?;

    // Initialize the SOL staker stake account.

    let mut data = ctx.accounts.sol_staker_stake.try_borrow_mut_data()?;
    let stake = bytemuck::from_bytes_mut::<SolStakerStake>(&mut data);

    *stake = SolStakerStake::new(withdrawer, *ctx.accounts.stake_state.key, validator_vote);
    stake.lamports_amount = u64::from(stake_state_data.activating)
        .checked_add(stake_state_data.effective.into())
        .ok_or(ProgramError::ArithmeticOverflow)?;

    // Update the validator stake account to increment the total SOL staked.

    validator_stake.total_staked_lamports_amount = validator_stake
        .total_staked_lamports_amount
        .checked_add(stake.lamports_amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    Ok(())
}
