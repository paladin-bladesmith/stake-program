use arrayref::array_ref;
use paladin_sol_stake_view_program_client::{
    instructions::GetStakeActivatingAndDeactivatingCpiBuilder,
    GetStakeActivatingAndDeactivatingReturnData,
};
use solana_program::{
    entrypoint::ProgramResult,
    program::{get_return_data, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};
use spl_discriminator::SplDiscriminate;

use crate::{
    err,
    error::StakeError,
    instruction::accounts::{Context, InitializeSolStakerStakeAccounts},
    processor::{unpack_initialized, unpack_initialized_mut},
    require,
    state::{
        find_sol_staker_authority_override_pda, find_sol_staker_stake_pda,
        find_validator_stake_pda, get_sol_staker_stake_pda_signer_seeds, Config, Delegation,
        SolStakerStake, ValidatorStake,
    },
};

/// Initializes stake account data for a SOL staker.
///
/// There can be only one SOL staker stake account per stake state and config account, since
/// the stake state is part of the SOL staker stake account seeds.
///
/// NOTE: Anybody can create the stake account for a SOL staker. For new
/// accounts, the authority is initialized to the stake state account's withdrawer.
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
    let config = ctx.accounts.config.try_borrow_data()?;
    let config = unpack_initialized::<Config>(&config)?;
    assert!(config.is_initialized());

    // stake state (validated by the SOL Stake View program)
    // - must have a delegation
    require!(
        ctx.accounts.sol_stake_view_program.key == &paladin_sol_stake_view_program_client::ID,
        ProgramError::IncorrectProgramId,
        "invalid sol stake view program"
    );
    GetStakeActivatingAndDeactivatingCpiBuilder::new(ctx.accounts.sol_stake_view_program)
        .stake(ctx.accounts.sol_staker_native_stake)
        .stake_history(ctx.accounts.sysvar_stake_history)
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
            return err!(StakeError::UndelegatedSolStakeAccount);
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

    // Validator vote must match the stake state account's validator vote (validation
    // done on the derivation of the expected address).
    let (derivation, _) =
        find_validator_stake_pda(&validator_vote, ctx.accounts.config.key, program_id);
    require!(
        ctx.accounts.validator_stake.key == &derivation,
        ProgramError::InvalidSeeds,
        "validator stake",
    );
    let mut stake_data = ctx.accounts.validator_stake.try_borrow_mut_data()?;
    let validator_stake = unpack_initialized_mut::<ValidatorStake>(&mut stake_data)?;

    // Sol staker stake
    // - Have the correct PDA derivation.
    // - Be uninitialized (empty data).
    //
    // NOTE: The stake account is created and assigned to the stake program, so it needs
    // to be pre-funded with the minimum rent balance by the caller.
    let (sol_staker_stake_key, sol_staker_stake_bump) = find_sol_staker_stake_pda(
        ctx.accounts.sol_staker_native_stake.key,
        ctx.accounts.config.key,
        program_id,
    );
    require!(
        ctx.accounts.sol_staker_stake.key == &sol_staker_stake_key,
        ProgramError::InvalidSeeds,
        "stake"
    );
    require!(
        ctx.accounts.sol_staker_stake.data_is_empty(),
        ProgramError::AccountAlreadyInitialized,
        "stake"
    );

    // Sol staker authority override.
    // - Correct derivation.
    // - Allowed to be uninitialized.
    let (sol_staker_authority_override, _) =
        find_sol_staker_authority_override_pda(&withdrawer, ctx.accounts.config.key, program_id);
    require!(
        &sol_staker_authority_override == ctx.accounts.sol_staker_authority_override.key,
        ProgramError::InvalidSeeds,
        "sol staker authority"
    );
    let sol_staker_authority_override = ctx.accounts.sol_staker_authority_override.data.borrow();
    let sol_staker_authority_override =
        match ctx.accounts.sol_staker_authority_override.data_is_empty() {
            true => None,
            false => Some(Pubkey::new_from_array(*array_ref![
                sol_staker_authority_override,
                0,
                32
            ])),
        };

    // Ensure the account is rent exempt.
    require!(
        ctx.accounts.sol_staker_stake.lamports()
            >= Rent::get()?.minimum_balance(SolStakerStake::LEN),
        ProgramError::AccountNotRentExempt,
        "sol staker stake",
    );

    // Allocate and assign.
    let bump_seed = [sol_staker_stake_bump];
    let signer_seeds = get_sol_staker_stake_pda_signer_seeds(
        ctx.accounts.sol_staker_native_stake.key,
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
    *stake = SolStakerStake {
        _discriminator: SolStakerStake::SPL_DISCRIMINATOR.into(),
        delegation: Delegation {
            staked_amount: 0,
            effective_amount: 0,
            unstake_cooldown: 0,
            authority: sol_staker_authority_override.unwrap_or(withdrawer),
            validator_vote,
            // NB: Will be set on the first stake.
            last_seen_holder_rewards_per_token: 0.into(),
            // NB: Will be set on the first stake.
            last_seen_stake_rewards_per_token: 0.into(),
        },
        lamports_amount: stake_state_data.effective.into(),
        sol_stake: *ctx.accounts.sol_staker_native_stake.key,
    };

    // Update the validator stake account to increment the total SOL staked.
    validator_stake.total_staked_lamports_amount = validator_stake
        .total_staked_lamports_amount
        .checked_add(stake.lamports_amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    Ok(())
}
