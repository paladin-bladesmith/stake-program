use arrayref::array_ref;
use solana_program::{
    entrypoint::ProgramResult, program::invoke_signed, program_error::ProgramError, pubkey::Pubkey,
    system_instruction, vote::state::VoteState,
};

use crate::{
    instruction::accounts::{Context, InitializeValidatorStakeAccounts},
    require,
    state::{
        find_validator_stake_pda, get_validator_stake_pda_signer_seeds, Config, ValidatorStake,
    },
};

/// Initializes stake account data for a validator.
///
/// NOTE: Anybody can create the stake account for a validator. For new
/// accounts, the authority is initialized to the validator vote account's
/// withdraw authority.
///
/// 0. `[ ]` Stake config account
/// 1. `[w]` Validator stake account
///     * PDA seeds: ['stake::state::validator_stake', validator, config]
/// 2. `[ ]` Validator vote account
/// 3. `[ ]` System program
pub fn process_initialize_validator_stake(
    program_id: &Pubkey,
    ctx: Context<InitializeValidatorStakeAccounts>,
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

    // validator_vote
    // - owner must be the vote program
    // - must be initialized

    require!(
        ctx.accounts.validator_vote.owner == &solana_program::vote::program::ID,
        ProgramError::InvalidAccountOwner,
        "validator_vote"
    );

    let data = &ctx.accounts.validator_vote.try_borrow_data()?;

    require!(
        VoteState::is_correct_size_and_initialized(data),
        ProgramError::InvalidAccountData,
        "validator_vote"
    );

    let withdraw_authority = Pubkey::from(*array_ref!(data, 36, 32));

    // stake
    // - have the correct PDA derivation
    // - be uninitialized (empty data)
    //
    // NOTE: The stake account is created and assigned to the stake program, so it needs
    // to be pre-funded with the minimum rent balance by the caller.
    let (derivation, bump) = find_validator_stake_pda(
        ctx.accounts.validator_vote.key,
        ctx.accounts.config.key,
        program_id,
    );
    require!(
        ctx.accounts.validator_stake.key == &derivation,
        ProgramError::InvalidSeeds,
        "stake"
    );
    require!(
        ctx.accounts.validator_stake.data_is_empty(),
        ProgramError::AccountAlreadyInitialized,
        "stake"
    );

    // Allocate and assign.
    let bump_seed = [bump];
    let signer_seeds = get_validator_stake_pda_signer_seeds(
        ctx.accounts.validator_vote.key,
        ctx.accounts.config.key,
        &bump_seed,
    );
    invoke_signed(
        &system_instruction::allocate(ctx.accounts.validator_stake.key, ValidatorStake::LEN as u64),
        &[ctx.accounts.validator_stake.clone()],
        &[&signer_seeds],
    )?;
    invoke_signed(
        &system_instruction::assign(ctx.accounts.validator_stake.key, program_id),
        &[ctx.accounts.validator_stake.clone()],
        &[&signer_seeds],
    )?;

    // Initialize the stake account.
    let mut data = ctx.accounts.validator_stake.try_borrow_mut_data()?;
    let validator_stake = bytemuck::from_bytes_mut::<ValidatorStake>(&mut data);
    *validator_stake = ValidatorStake::new(withdraw_authority, *ctx.accounts.validator_vote.key);

    Ok(())
}
