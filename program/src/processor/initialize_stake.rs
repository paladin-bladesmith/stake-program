use arrayref::array_ref;
use solana_program::{
    entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey, vote::state::VoteState,
};
use spl_discriminator::SplDiscriminate;

use crate::{
    error::StakeError,
    instruction::accounts::{Context, InitializeStakeAccounts},
    require,
    state::{find_stake_pda, Config, Stake},
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

    let data = &ctx.accounts.config.try_borrow_data()?;

    require!(
        VoteState::is_correct_size_and_initialized(data),
        ProgramError::InvalidAccountData,
        "validator_vote"
    );

    let validator = Pubkey::from(*array_ref!(data, 0, 32));
    let withdraw_authority = Pubkey::from(*array_ref!(data, 32, 32));

    // 3. stake
    // - owner must be stake program
    // - have the correct PDA derivation
    // - have the correct length
    // - be uninitialized

    require!(
        ctx.accounts.stake.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "stake"
    );

    let (derivation, _) = find_stake_pda(&validator, ctx.accounts.config.key, program_id);

    require!(
        ctx.accounts.stake.key == &derivation,
        ProgramError::InvalidSeeds,
        "stake"
    );

    let mut data = ctx.accounts.config.try_borrow_mut_data()?;

    require!(
        data.len() == Stake::LEN,
        StakeError::InvalidAccountDataLength,
        "stake"
    );

    let stake = bytemuck::from_bytes_mut::<Stake>(&mut data);

    require!(
        stake.is_uninitialized(),
        ProgramError::AccountAlreadyInitialized,
        "stake"
    );

    // Initialize the stake account.

    *stake = Stake {
        discriminator: Stake::SPL_DISCRIMINATOR.into(),
        validator,
        authority: withdraw_authority,
        ..Stake::default()
    };

    Ok(())
}
