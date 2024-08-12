use solana_program::{
    clock::Clock, entrypoint::ProgramResult, msg, program_error::ProgramError, pubkey::Pubkey,
    sysvar::Sysvar,
};

use crate::{
    error::StakeError,
    instruction::accounts::{Context, InactivateSolStakerStakeAccounts},
    processor::unpack_initialized_mut,
    require,
    state::{
        find_sol_staker_stake_pda, find_validator_stake_pda, Config, SolStakerStake, ValidatorStake,
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
/// 2. `[w]` Validator stake account
pub fn process_inactivate_sol_staker_stake(
    program_id: &Pubkey,
    ctx: Context<InactivateSolStakerStakeAccounts>,
) -> ProgramResult {
    // Account validation.

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

    // stake
    // - owner must be the stake program
    // - must be a SolStakerStake account
    // - must be initialized
    // - must have the correct derivation

    require!(
        ctx.accounts.stake.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "stake"
    );

    let data = &mut ctx.accounts.stake.try_borrow_mut_data()?;
    let sol_staker_stake = unpack_initialized_mut::<SolStakerStake>(data)?;

    let (derivation, _) = find_sol_staker_stake_pda(
        &sol_staker_stake.sol_stake,
        ctx.accounts.config.key,
        program_id,
    );

    require!(
        ctx.accounts.stake.key == &derivation,
        ProgramError::InvalidSeeds,
        "stake",
    );

    let delegation = &mut sol_staker_stake.delegation;

    // validator stake
    // - owner must be the stake program
    // - must be a ValidatorStake account
    // - must be initialized
    // - must have the correct derivation

    require!(
        ctx.accounts.stake.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "validator stake"
    );

    let data = &mut ctx.accounts.stake.try_borrow_mut_data()?;
    let validator_stake = unpack_initialized_mut::<ValidatorStake>(data)?;

    let (derivation, _) = find_validator_stake_pda(
        // validates the sol staker stake <-> validator stake relationship
        &delegation.validator_vote,
        ctx.accounts.config.key,
        program_id,
    );

    require!(
        ctx.accounts.validator_stake.key == &derivation,
        ProgramError::InvalidSeeds,
        "validator stake",
    );

    // Inactivates the stake if elegible.

    if let Some(timestamp) = delegation.deactivation_timestamp {
        let inactive_timestamp = config.cooldown_time_seconds.saturating_add(timestamp.get());
        let current_timestamp = Clock::get()?.unix_timestamp as u64;

        require!(
            current_timestamp >= inactive_timestamp,
            StakeError::ActiveDeactivationCooldown,
            "{} second(s) remaining for deactivation",
            inactive_timestamp.saturating_sub(current_timestamp),
        );

        msg!("Inactivating {} token(s)", delegation.deactivating_amount);

        // moves deactivating amount to inactive
        delegation.amount = delegation
            .amount
            .checked_sub(delegation.deactivating_amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        delegation.inactive_amount = delegation
            .inactive_amount
            .checked_add(delegation.deactivating_amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        // Update the validator stake account data.

        validator_stake.total_staked_token_amount = validator_stake
            .total_staked_token_amount
            .checked_sub(delegation.deactivating_amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        // Update the config account data.

        config.token_amount_delegated = config
            .token_amount_delegated
            .checked_sub(delegation.deactivating_amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        // Clears the deactivation.

        delegation.deactivating_amount = 0;
        delegation.deactivation_timestamp = None;

        Ok(())
    } else {
        Err(StakeError::NoDeactivatedTokens.into())
    }
}
