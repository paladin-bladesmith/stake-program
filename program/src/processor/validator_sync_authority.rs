use arrayref::array_ref;
use solana_program::{
    entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey, vote::state::VoteState,
};

use crate::{
    instruction::accounts::{Context, ValidatorSyncAuthorityAccounts},
    processor::unpack_initialized_mut,
    require,
    state::{find_validator_stake_pda, Config, ValidatorStake},
};

pub(crate) fn process_validator_sync_authority(
    program_id: &Pubkey,
    ctx: Context<ValidatorSyncAuthorityAccounts>,
) -> ProgramResult {
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

    // stake
    // - have the correct PDA derivation
    // - be initialized
    let (derivation, _) = find_validator_stake_pda(
        ctx.accounts.validator_vote.key,
        ctx.accounts.config.key,
        program_id,
    );
    require!(
        ctx.accounts.validator_stake.key == &derivation,
        ProgramError::InvalidSeeds,
        "stake"
    );
    let mut validator_stake = ctx.accounts.validator_stake.data.borrow_mut();
    let validator_stake = unpack_initialized_mut::<ValidatorStake>(&mut validator_stake).unwrap();

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

    // Sync the authority to match the current withdraw authority.
    validator_stake.delegation.authority = withdraw_authority;

    Ok(())
}
