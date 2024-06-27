use solana_program::{entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey};
use spl_pod::optional_keys::OptionalNonZeroPubkey;

use crate::{
    error::StakeError,
    instruction::{
        accounts::{Context, SetAuthorityAccounts},
        AuthorityType,
    },
    require,
    state::{Config, Stake},
};

/// Sets new authority on a config or stake account
///
/// ### Accounts:
///
///   0. `[w]` config
///   1. `[s]` config_authority
///   2. `[]` new_authority
pub fn process_set_authority(
    program_id: &Pubkey,
    ctx: Context<SetAuthorityAccounts>,
    authority_type: AuthorityType,
) -> ProgramResult {
    // Accounts validation.

    // 1. authority
    // - must be a signer
    // - must match the authority in the account (checked in the match statement below)

    require!(
        ctx.accounts.authority.is_signer,
        ProgramError::MissingRequiredSignature,
        "authority"
    );

    // 2. account
    // - owner must the stake program
    // - must be initialized
    // - must have an authority set

    require!(
        ctx.accounts.account.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "account"
    );

    let data = &mut ctx.accounts.account.try_borrow_mut_data()?;

    match authority_type {
        AuthorityType::Config | AuthorityType::Slash => {
            let config = bytemuck::try_from_bytes_mut::<Config>(data)
                .map_err(|_error| ProgramError::InvalidAccountData)?;

            require!(
                config.is_initialized(),
                ProgramError::UninitializedAccount,
                "config"
            );

            // Asserts that the authority is set - only updates the authority jf there is
            // one set and it matches the signing authority.
            //
            // TODO: do we fail if the authority is not set?

            let current_authority = config.authority.into();

            if let Some(current_authority) = current_authority {
                require!(
                    *ctx.accounts.authority.key == current_authority,
                    StakeError::InvalidAuthority,
                    "authority (config)"
                );

                match authority_type {
                    AuthorityType::Config => {
                        config.authority = OptionalNonZeroPubkey(*ctx.accounts.new_authority.key)
                    }
                    AuthorityType::Slash => {
                        config.slash_authority =
                            OptionalNonZeroPubkey(*ctx.accounts.new_authority.key)
                    }
                    _ => (), /* unreachable */
                }
            }
        }
        AuthorityType::Stake => {
            let stake = bytemuck::try_from_bytes_mut::<Stake>(data)
                .map_err(|_error| ProgramError::InvalidAccountData)?;

            require!(
                *ctx.accounts.authority.key == stake.authority,
                StakeError::InvalidAuthority,
                "authority (stake)"
            );

            stake.authority = *ctx.accounts.new_authority.key;
        }
    }

    Ok(())
}
