use solana_program::{entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey};

use crate::{
    err,
    error::StakeError,
    instruction::{
        accounts::{Context, UpdateConfigAccounts},
        ConfigField,
    },
    processor::unpack_initialized_mut,
    require,
    state::{Config, MAX_BASIS_POINTS},
};

/// Updates configuration parameters.
///
/// ### Accounts:
///
///   0. `[w]` Stake config account
///   1. `[s]` Stake config authority
pub fn process_update_config(
    program_id: &Pubkey,
    ctx: Context<UpdateConfigAccounts>,
    field: ConfigField,
) -> ProgramResult {
    // Accounts validation.

    // config
    // - owner must be the stake program
    // - must be initialized
    require!(
        ctx.accounts.config.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "config"
    );
    let mut config = ctx.accounts.config.try_borrow_mut_data()?;
    let config = unpack_initialized_mut::<Config>(&mut config)?;

    let authority: Option<Pubkey> = config.authority.into();
    if let Some(authority) = authority {
        // config_authority
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
