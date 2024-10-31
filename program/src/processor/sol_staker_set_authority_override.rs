use arrayref::array_mut_ref;
use solana_program::{
    entrypoint::ProgramResult, program::invoke_signed, program_error::ProgramError, pubkey::Pubkey,
    system_instruction,
};

use crate::{
    err,
    error::StakeError,
    instruction::accounts::{Context, SolStakerSetAuthorityOverrideAccounts},
    processor::unpack_initialized,
    require,
    state::{find_sol_staker_authority_override_pda, Config},
};

pub(crate) fn process_sol_staker_set_authority_override(
    program_id: &Pubkey,
    ctx: Context<SolStakerSetAuthorityOverrideAccounts>,
    authority_original: Pubkey,
    authority_override: Pubkey,
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

    // Sol staker authority override.
    // - Must match PDA(config, authority).
    let (sol_stake_authority_key, sol_staker_authority_override_bump) =
        find_sol_staker_authority_override_pda(
            &authority_original,
            ctx.accounts.config.key,
            program_id,
        );
    require!(
        &sol_stake_authority_key == ctx.accounts.sol_staker_authority_override.key,
        ProgramError::InvalidSeeds,
        "sol staker authority override",
    );

    // Initialize the account if necessary (this assumes the caller has pre-funded rent).
    if ctx.accounts.sol_staker_authority_override.owner != program_id {
        // Allocate the required space.
        invoke_signed(
            &system_instruction::allocate(ctx.accounts.sol_staker_authority_override.key, 32),
            &[ctx.accounts.sol_staker_authority_override.clone()],
            &[&[
                &authority_original.to_bytes() as &[u8],
                &[sol_staker_authority_override_bump],
            ]],
        )?;

        // Set the funnel program as the owner.
        invoke_signed(
            &system_instruction::assign(
                ctx.accounts.sol_staker_authority_override.key,
                &program_id,
            ),
            &[ctx.accounts.sol_staker_authority_override.clone()],
            &[&[
                &authority_original.to_bytes() as &[u8],
                &[sol_staker_authority_override_bump],
            ]],
        )?;
    }

    // Set the override.
    let mut key = ctx.accounts.sol_staker_authority_override.data.borrow_mut();
    let key = array_mut_ref![key, 0, 32];
    *key = authority_override.to_bytes();

    Ok(())
}
