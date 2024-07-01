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
///   0. `[w]` config or stake account
///   1. `[s]` current authority
///   2. `[]` new authority
pub fn process_set_authority(
    program_id: &Pubkey,
    ctx: Context<SetAuthorityAccounts>,
    authority_type: AuthorityType,
) -> ProgramResult {
    // Accounts validation.

    // 1. authority
    // - must be a signer
    // - must match the authority on the account (checked in the match statement below)

    require!(
        ctx.accounts.authority.is_signer,
        ProgramError::MissingRequiredSignature,
        "authority"
    );

    // 2. account
    // - owner must the stake program
    // - must be initialized
    // - must have an authority set
    // - current authority must match the signing authority

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

            // Asserts whether the authority is set or not - only updates the authority if
            // there is one set and it matches the signing authority.

            match authority_type {
                AuthorityType::Config => {
                    let config_authority =
                        <OptionalNonZeroPubkey as Into<Option<Pubkey>>>::into(config.authority)
                            .ok_or(StakeError::InvalidAuthority)?;

                    require!(
                        *ctx.accounts.authority.key == config_authority,
                        StakeError::InvalidAuthority,
                        "authority (config)"
                    );

                    config.authority = OptionalNonZeroPubkey(*ctx.accounts.new_authority.key)
                }
                AuthorityType::Slash => {
                    let slash_authority = <OptionalNonZeroPubkey as Into<Option<Pubkey>>>::into(
                        config.slash_authority,
                    )
                    .ok_or(StakeError::InvalidAuthority)?;

                    require!(
                        *ctx.accounts.authority.key == slash_authority,
                        StakeError::InvalidAuthority,
                        "authority (slash)"
                    );

                    config.slash_authority = OptionalNonZeroPubkey(*ctx.accounts.new_authority.key);
                }
                _ => (), /* unreachable */
            }
        }
        AuthorityType::Stake => {
            let stake = bytemuck::try_from_bytes_mut::<Stake>(data)
                .map_err(|_error| ProgramError::InvalidAccountData)?;

            require!(
                stake.is_initialized(),
                ProgramError::UninitializedAccount,
                "stake"
            );

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
