use solana_program::{entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey};

use crate::{
    error::StakeError,
    instruction::{
        accounts::{Context, UpdateConfigAccounts},
        ConfigField,
    },
    require,
    state::{AccountType, Config},
};

/// Updates configuration parameters.
///
/// TODO: is there any validation needed for the field values?
///
/// ### Accounts:
///
///   0. `[w]` config
///   1. `[s]` config_authority
pub fn process_update_config(
    program_id: &Pubkey,
    ctx: Context<UpdateConfigAccounts>,
    field: ConfigField,
) -> ProgramResult {
    // Accounts validation.

    require!(
        ctx.accounts.config.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "config"
    );

    let data = &mut ctx.accounts.config.try_borrow_mut_data()?;

    require!(
        !data.is_empty() && data[0] == AccountType::Config as u8,
        ProgramError::InvalidAccountData,
        "config"
    );

    let config = bytemuck::from_bytes_mut::<Config>(data);

    require!(
        &config.authority.0 == ctx.accounts.config_authority.key,
        StakeError::InvalidAuthority,
        "config_authority"
    );

    require!(
        ctx.accounts.config_authority.is_signer,
        ProgramError::MissingRequiredSignature,
        "config_authority"
    );

    // Updates the config account.

    match field {
        ConfigField::CooldownTimeSecs(seconds) => {
            config.cooldown_time_seconds = seconds.into();
        }
        ConfigField::MaxDeactivationBasisPoints(points) => {
            config.max_deactivation_basis_points = points.into();
        }
    }

    Ok(())
}
