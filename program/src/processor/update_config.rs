use solana_program::{entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey};

use crate::{
    err,
    error::StakeError,
    instruction::{
        accounts::{Context, UpdateConfigAccounts},
        ConfigField,
    },
    require,
    state::{Config, MAX_BASIS_POINTS},
};

/// Updates configuration parameters.
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

    // 1. config
    // - owner must the stake program
    // - must be initialized

    require!(
        ctx.accounts.config.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "config"
    );

    let data = &mut ctx.accounts.config.try_borrow_mut_data()?;

    let config = bytemuck::try_from_bytes_mut::<Config>(data)
        .map_err(|_error| ProgramError::InvalidAccountData)?;

    require!(
        config.is_initialized(),
        ProgramError::UninitializedAccount,
        "config"
    );

    let authority: Option<Pubkey> = config.authority.into();

    if let Some(authority) = authority {
        // 2. config_authority
        // - config_authority must match the authority in the config
        // - must be a signer

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

        // Updates the config account.

        match field {
            ConfigField::CooldownTimeSeconds(seconds) => {
                config.cooldown_time_seconds = seconds;
            }
            ConfigField::MaxDeactivationBasisPoints(points) => {
                require!(
                    points <= MAX_BASIS_POINTS as u16,
                    ProgramError::InvalidArgument,
                    "basis points exceeds maximum allowed value of {}",
                    MAX_BASIS_POINTS
                );

                config.max_deactivation_basis_points = points;
            }
            ConfigField::SyncRewardsLamports(lamports) => {
                config.sync_rewards_lamports = lamports;
            }
        }
    } else {
        return err!(StakeError::AuthorityNotSet);
    }

    Ok(())
}
