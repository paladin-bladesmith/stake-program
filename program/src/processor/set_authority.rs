use solana_program::{entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey};
use spl_pod::optional_keys::OptionalNonZeroPubkey;

use crate::{
    error::StakeError,
    instruction::{
        accounts::{Context, SetAuthorityAccounts},
        AuthorityType,
    },
    processor::unpack_initialized_mut,
    require,
    state::Config,
};

/// Sets new authority on a config or stake account.
///
/// ### Accounts:
///
///   0. `[w]` Config or Stake account
///   1. `[s]` Current authority on the account
///   2. `[ ]` Authority to set
pub fn process_set_authority(
    program_id: &Pubkey,
    ctx: Context<SetAuthorityAccounts>,
    authority_type: AuthorityType,
) -> ProgramResult {
    // Accounts validation.

    // authority
    // - must be a signer
    // - must match the authority on the account (checked in the match statement below)
    require!(
        ctx.accounts.authority.is_signer,
        ProgramError::MissingRequiredSignature,
        "authority"
    );

    // account
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
        AuthorityType::Config => {
            let config = unpack_initialized_mut::<Config>(data)?;
            let config_authority =
                Option::<Pubkey>::from(config.authority).ok_or(StakeError::AuthorityNotSet)?;
            require!(
                *ctx.accounts.authority.key == config_authority,
                StakeError::InvalidAuthority,
                "authority (config)"
            );

            config.authority = OptionalNonZeroPubkey(*ctx.accounts.new_authority.key)
        }
        AuthorityType::Slash => {
            let config = unpack_initialized_mut::<Config>(data)?;
            let slash_authority = Option::<Pubkey>::from(config.slash_authority)
                .ok_or(StakeError::AuthorityNotSet)?;
            require!(
                *ctx.accounts.authority.key == slash_authority,
                StakeError::InvalidAuthority,
                "authority (slash)"
            );

            config.slash_authority = OptionalNonZeroPubkey(*ctx.accounts.new_authority.key);
        }
    }

    Ok(())
}
