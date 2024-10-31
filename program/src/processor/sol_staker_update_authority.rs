use solana_program::{entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey};

use crate::{
    err,
    error::StakeError,
    instruction::accounts::{Context, SolStakerUpdateAuthorityAccounts},
    processor::{unpack_initialized, unpack_initialized_mut},
    require,
    state::{find_sol_staker_stake_pda, Config, SolStakerStake},
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
    let config = unpack_initialized::<Config>(&config)?;

    // Config Authority.
    // - Must match the provided authority account.
    // - Must sign the transaction.
    let authority = config.authority.0;
    if authority == Pubkey::default() {
        return err!(StakeError::AuthorityNotSet);
    };
    require!(
        ctx.accounts.config_authority.key == &authority,
        StakeError::InvalidAuthority,
        "config_authority"
    );
    require!(
        ctx.accounts.config_authority.is_signer,
        ProgramError::MissingRequiredSignature,
        "config_authority"
    );

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
    // - Must not be default pubkey (all zeroes).
    require!(
        ctx.accounts.sol_staker_authority_override.owner == program_id,
        ProgramError::IllegalOwner,
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
