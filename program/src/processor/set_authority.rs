use solana_program::{entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey};
use spl_pod::optional_keys::OptionalNonZeroPubkey;

use crate::{
    error::StakeError,
    instruction::{
        accounts::{Context, SetAuthorityAccounts},
        AuthorityType,
    },
    require,
    state::{AccountType, Config, Stake},
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

    require!(
        ctx.accounts.account.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "account"
    );

    require!(
        ctx.accounts.authority.is_signer,
        ProgramError::MissingRequiredSignature,
        "authority"
    );

    let data = &mut ctx.accounts.account.try_borrow_mut_data()?;

    require!(
        !data.is_empty(),
        ProgramError::UninitializedAccount,
        "account"
    );

    match authority_type {
        AuthorityType::Config | AuthorityType::Slash => {
            require!(
                AccountType::Config == data[0].into(),
                ProgramError::InvalidAccountData,
                "expected stake account, found {:?}",
                <u8 as Into<AccountType>>::into(data[0])
            );

            let config = bytemuck::from_bytes_mut::<Config>(data);

            // Asserts that the authority is set - only updates the authority there is one set
            // and it is a signer.
            //
            // TODO: do we fail is the authority is not set?

            if let Some(current_authority) =
                <OptionalNonZeroPubkey as Into<Option<Pubkey>>>::into(config.authority)
            {
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
            require!(
                AccountType::Stake == data[0].into(),
                ProgramError::InvalidAccountData,
                "expected stake account, found {:?}",
                <u8 as Into<AccountType>>::into(data[0])
            );

            let stake = bytemuck::from_bytes_mut::<Stake>(data);

            require!(
                *ctx.accounts.authority.key == stake.authority,
                StakeError::InvalidAuthority,
                "authority (stakes)"
            );

            stake.authority = *ctx.accounts.new_authority.key;
        }
    }

    Ok(())
}
