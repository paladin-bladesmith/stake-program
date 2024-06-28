use {
    crate::{
        error::StakeError,
        instruction::{
            accounts::{Context, UpdateConfigAccounts},
            ConfigField,
        },
        require,
        state::Config,
    },
    solana_program::{
        clock::UnixTimestamp, entrypoint::ProgramResult, program_error::ProgramError,
        pubkey::Pubkey,
    },
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
                config.cooldown_time_seconds = seconds as UnixTimestamp;
            }
            ConfigField::MaxDeactivationBasisPoints(points) => {
                config.max_deactivation_basis_points = points;
            }
        }
    } else {
        // TODO: do we need to log a message to say that the config authority is
        // not set and the update did not happen? Or fail the
        // transaction?
    }

    Ok(())
}
