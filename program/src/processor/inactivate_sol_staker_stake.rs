use solana_program::{
    clock::Clock, entrypoint::ProgramResult, msg, program_error::ProgramError, pubkey::Pubkey,
    sysvar::Sysvar,
};

use crate::{
    error::StakeError,
    instruction::accounts::{Context, InactivateSolStakerStakeAccounts},
    processor::{harvest, unpack_initialized_mut, HarvestAccounts},
    require,
    state::{
        calculate_maximum_stake_for_lamports_amount, find_sol_staker_stake_pda, Config,
        SolStakerStake,
    },
};

/// Move tokens from deactivating to inactive.
///
/// Reduces the total voting power for the SOL staker stake account, the total staked
/// amount on the corresponding validator stake and config accounts.
///
/// NOTE: This instruction is permissionless, so anybody can finish
/// deactivating someone's tokens, preparing them to be withdrawn.
///
/// 0. `[w]` Stake config account
/// 1. `[w]` SOL staker stake account
pub fn process_inactivate_sol_staker_stake(
    program_id: &Pubkey,
    ctx: Context<InactivateSolStakerStakeAccounts>,
) -> ProgramResult {
    // Account validation.

    // stake
    // - owner must be the stake program
    // - must be a SolStakerStake account
    // - must be initialized
    // - must have the correct derivation
    require!(
        ctx.accounts.sol_staker_stake.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "stake"
    );
    let data = &mut ctx.accounts.sol_staker_stake.try_borrow_mut_data()?;
    let sol_staker_stake = unpack_initialized_mut::<SolStakerStake>(data)?;
    let (derivation, _) = find_sol_staker_stake_pda(
        &sol_staker_stake.sol_stake,
        ctx.accounts.config.key,
        program_id,
    );
    require!(
        ctx.accounts.sol_staker_stake.key == &derivation,
        ProgramError::InvalidSeeds,
        "stake",
    );
    let delegation = &mut sol_staker_stake.delegation;

    // Harvest rewards & update last claim tracking.
    harvest(
        HarvestAccounts {
            config: ctx.accounts.config,
            holder_rewards: ctx.accounts.vault_holder_rewards,
            recipient: ctx.accounts.sol_staker_stake_authority,
        },
        delegation,
        None,
    )?;

    // config
    // - owner must be the stake program
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
        "config",
    );

    // Inactivates the stake if elegible.
    let Some(timestamp) = delegation.deactivation_timestamp else {
        return Err(StakeError::NoDeactivatedTokens.into());
    };
    let inactive_timestamp = config.cooldown_time_seconds.saturating_add(timestamp.get());
    let current_timestamp = Clock::get()?.unix_timestamp as u64;

    require!(
        current_timestamp >= inactive_timestamp,
        StakeError::ActiveDeactivationCooldown,
        "{} second(s) remaining for deactivation",
        inactive_timestamp.saturating_sub(current_timestamp),
    );

    msg!("Inactivating {} token(s)", delegation.deactivating_amount);

    // Compute the new stake values.
    let staker_active = delegation
        .active_amount
        .checked_sub(delegation.deactivating_amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    let staker_inactive = delegation
        .inactive_amount
        .checked_add(delegation.deactivating_amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    let staker_limit =
        calculate_maximum_stake_for_lamports_amount(sol_staker_stake.lamports_amount)?;
    let staker_effective = std::cmp::min(staker_active, staker_limit);
    let effective_delta = delegation
        .effective_amount
        .checked_sub(staker_effective)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    // Update the state values.
    delegation.active_amount = staker_active;
    delegation.effective_amount = staker_effective;
    delegation.deactivating_amount = 0;
    delegation.deactivation_timestamp = None;
    delegation.inactive_amount = staker_inactive;
    config.token_amount_effective = config
        .token_amount_effective
        .checked_sub(effective_delta)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    Ok(())
}
