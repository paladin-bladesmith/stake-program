use solana_program::{entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey};

use crate::{
    err,
    error::StakeError,
    instruction::accounts::{Context, ValidatorOverrideStakedLamportsAccounts},
    processor::{harvest, sync_effective, unpack_initialized_mut, HarvestAccounts},
    require,
    state::{find_validator_stake_pda, find_vault_pda, Config, ValidatorStake},
};

pub(crate) fn process_validator_override_staked_lamports(
    program_id: &Pubkey,
    ctx: Context<ValidatorOverrideStakedLamportsAccounts>,
    amount_min: u64,
) -> ProgramResult {
    // Config
    // - Owner must be this program.
    // - Must be initialized.
    require!(
        ctx.accounts.config.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "config"
    );
    let mut config = ctx.accounts.config.data.borrow_mut();
    let config = unpack_initialized_mut::<Config>(&mut config)?;

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

    // validator stake
    // - owner must be the stake program
    // - must be initialized
    // - must have the correct derivation
    require!(
        ctx.accounts.validator_stake.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "validator stake"
    );
    let mut stake_data = ctx.accounts.validator_stake.try_borrow_mut_data()?;
    let stake = unpack_initialized_mut::<ValidatorStake>(&mut stake_data)?;
    let (derivation, _) = find_validator_stake_pda(
        &stake.delegation.validator_vote,
        ctx.accounts.config.key,
        program_id,
    );
    require!(
        ctx.accounts.validator_stake.key == &derivation,
        ProgramError::InvalidSeeds,
        "validator stake",
    );

    // Harvest rewards & update last claim tracking.
    let vault_authority = find_vault_pda(ctx.accounts.config.key, program_id).0;
    harvest(
        HarvestAccounts {
            config: ctx.accounts.config,
            vault_holder_rewards: ctx.accounts.vault_holder_rewards,
            authority: ctx.accounts.validator_stake_authority,
        },
        config,
        &vault_authority,
        &mut stake.delegation,
        None,
    )?;

    // Update the override minimum.
    stake.total_staked_lamports_amount_min = amount_min;

    // Sync the effective stake.
    sync_effective(
        config,
        &mut stake.delegation,
        (
            stake.total_staked_lamports_amount,
            stake.total_staked_lamports_amount_min,
        ),
    )?;

    Ok(())
}
