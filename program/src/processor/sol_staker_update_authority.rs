use solana_program::{entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey};

use crate::{
    instruction::accounts::{Context, SolStakerUpdateAuthorityAccounts},
    processor::{unpack_initialized, unpack_initialized_mut},
    require,
    state::{
        find_sol_staker_authority_override_pda, find_sol_staker_stake_pda, Config, SolStakerStake,
    },
};

pub(crate) fn process_sol_staker_update_authority(
    program_id: &Pubkey,
    ctx: Context<SolStakerUpdateAuthorityAccounts>,
) -> ProgramResult {
    // Config
    // - Owner must be this program.
    // - Must be initialized.
    require!(
        ctx.accounts.config.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "config"
    );
    let config = ctx.accounts.config.data.borrow();
    let _ = unpack_initialized::<Config>(&config)?;

    // Sol staker stake.
    // - Must be owned by this program.
    require!(
        ctx.accounts.sol_staker_stake.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "sol staker stake"
    );
    let mut sol_staker_stake = ctx.accounts.sol_staker_stake.data.borrow_mut();
    let sol_staker_stake = unpack_initialized_mut::<SolStakerStake>(&mut sol_staker_stake)?;
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

    // Sol staker authority override.
    // - Must be owned by this program.
    // - Must match authority override derivation.
    // - Must not be default pubkey (all zeroes).
    require!(
        ctx.accounts.sol_staker_authority_override.owner == program_id,
        ProgramError::IllegalOwner,
        "sol staker authority override"
    );
    let (derivation, _) = find_sol_staker_authority_override_pda(
        &sol_staker_stake.delegation.authority,
        ctx.accounts.config.key,
        program_id,
    );
    require!(
        &derivation == ctx.accounts.sol_staker_authority_override.key,
        ProgramError::InvalidSeeds,
        "sol staker authority override"
    );
    let authority_override = ctx.accounts.sol_staker_authority_override.data.borrow();
    let authority_override =
        Pubkey::new_from_array(*arrayref::array_ref![authority_override, 0, 32]);
    require!(
        authority_override != Pubkey::default(),
        ProgramError::UninitializedAccount,
        "sol staker authority override"
    );

    // Update the authority on the account.
    sol_staker_stake.delegation.authority = authority_override;

    Ok(())
}
