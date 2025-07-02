use arrayref::array_ref;
use solana_program::{
    entrypoint::ProgramResult, program::invoke_signed, program_error::ProgramError, pubkey::Pubkey,
    rent::Rent, system_instruction, sysvar::Sysvar, vote::state::VoteState,
};
use spl_discriminator::SplDiscriminate;

use crate::{
    error::StakeError,
    instruction::accounts::{Context, InitializeValidatorStakeAccounts},
    require,
    state::{
        find_duna_document_pda, find_validator_stake_pda, get_validator_stake_pda_signer_seeds,
        Config, Delegation, ValidatorStake,
    },
};

/// Initializes stake account data for a validator.
///
/// NOTE: Anybody can create the stake account for a validator. For new
/// accounts, the authority is initialized to the validator vote account's
/// withdraw authority.
///
/// 0. `[ ]` Stake config
/// 1. `[w]` Validator stake
/// 2. `[ ]` Validator vote
/// 3. `[ ]` System program
pub fn process_initialize_validator_stake(
    program_id: &Pubkey,
    ctx: Context<InitializeValidatorStakeAccounts>,
) -> ProgramResult {
    // Accounts validation.

    // config
    // - owner must be this program
    // - must be initialized
    require!(
        ctx.accounts.config.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "config"
    );
    let data = &ctx.accounts.config.try_borrow_data()?;
    let config =
        bytemuck::try_from_bytes::<Config>(data).map_err(|_| ProgramError::InvalidAccountData)?;
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

    // Get duna doc PDA
    let (duna_document_pda, _) =
        find_duna_document_pda(&withdraw_authority, &config.duna_document_hash);

    // Check the duna document PDA is correct.
    require!(
        ctx.accounts.duna_document_pda.key == &duna_document_pda,
        ProgramError::InvalidSeeds,
        "duna document"
    );

    // Ensure the duna document PDA is initialized.
    let duna_document_data = ctx.accounts.duna_document_pda.try_borrow_data()?;

    require!(
        duna_document_data.get(0) == Some(&1),
        StakeError::DunaDocumentNotInitialized,
        "duna document"
    );

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

    // Ensure the account is rent exempt.
    require!(
        ctx.accounts.validator_stake.lamports()
            >= Rent::get()?.minimum_balance(ValidatorStake::LEN),
        ProgramError::AccountNotRentExempt,
        "validator stake",
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
    *validator_stake = ValidatorStake {
        _discriminator: ValidatorStake::SPL_DISCRIMINATOR.into(),
        delegation: Delegation {
            staked_amount: 0,
            effective_amount: 0,
            unstake_cooldown: 0,
            authority: withdraw_authority,
            validator_vote: *ctx.accounts.validator_vote.key,
            // NB: Will be set on the first stake.
            last_seen_holder_rewards_per_token: 0.into(),
            // NB: Will be set on the first stake.
            last_seen_stake_rewards_per_token: 0.into(),
        },
        total_staked_lamports_amount: 0,
        total_staked_lamports_amount_min: 0,
    };

    Ok(())
}
