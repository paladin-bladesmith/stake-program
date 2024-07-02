use arrayref::array_ref;
use solana_program::{
    entrypoint::ProgramResult, program::invoke_signed, program_error::ProgramError, pubkey::Pubkey,
    system_instruction, vote::state::VoteState,
};
use spl_discriminator::SplDiscriminate;

use crate::{
    instruction::accounts::{Context, InitializeStakeAccounts},
    require,
    state::{find_stake_pda, get_stake_pda_signer_seeds, Config, Stake},
};

/// Initializes stake account data for a validator.
///
/// NOTE: Anybody can create the stake account for a validator. For new
/// accounts, the authority is initialized to the validator vote account's
/// withdraw authority.
///
/// 0. `[]` Stake config account
/// 1. `[w]` Validator stake account
///     * PDA seeds: ['stake', validator, config_account]
/// 2. `[]` Validator vote account
/// 3. `[]` System program
pub fn process_initialize_stake(
    program_id: &Pubkey,
    ctx: Context<InitializeStakeAccounts>,
) -> ProgramResult {
    // Accounts validation.

    // 1. config
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

    // 2. validator_vote
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

    let validator = Pubkey::from(*array_ref!(data, 4, 32));
    let withdraw_authority = Pubkey::from(*array_ref!(data, 36, 32));

    // 3. stake
    // - have the correct PDA derivation
    // - be uninitialized (empty data)
    //
    // NOTE: The stake account is created and assigned to the stake program, so it needs
    // to be pre-funded with the minimum rent balance by the caller.

    let (derivation, bump) = find_stake_pda(&validator, ctx.accounts.config.key, program_id);

    require!(
        ctx.accounts.stake.key == &derivation,
        ProgramError::InvalidSeeds,
        "stake"
    );

    require!(
        ctx.accounts.stake.data_is_empty(),
        ProgramError::AccountAlreadyInitialized,
        "stake"
    );

    // Allocate and assign.

    let bump_seed = [bump];
    let signer_seeds = get_stake_pda_signer_seeds(&validator, ctx.accounts.config.key, &bump_seed);

    invoke_signed(
        &system_instruction::allocate(ctx.accounts.stake.key, Stake::LEN as u64),
        &[ctx.accounts.stake.clone()],
        &[&signer_seeds],
    )?;

    invoke_signed(
        &system_instruction::assign(ctx.accounts.stake.key, program_id),
        &[ctx.accounts.stake.clone()],
        &[&signer_seeds],
    )?;

    // Initialize the stake account.

    let mut data = ctx.accounts.stake.try_borrow_mut_data()?;
    let stake = bytemuck::from_bytes_mut::<Stake>(&mut data);

    *stake = Stake {
        discriminator: Stake::SPL_DISCRIMINATOR.into(),
        validator,
        authority: withdraw_authority,
        ..Stake::default()
    };

    Ok(())
}
