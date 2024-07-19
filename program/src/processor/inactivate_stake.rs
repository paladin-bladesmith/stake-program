use solana_program::{
    clock::Clock, entrypoint::ProgramResult, msg, program_error::ProgramError, pubkey::Pubkey,
    sysvar::Sysvar,
};

use crate::{
    error::StakeError,
    instruction::accounts::{Context, InactivateStakeAccounts},
    require,
    state::{find_stake_pda, Config, Stake},
};

/// Move tokens from deactivating to inactive.
///
/// Reduces the total voting power for the stake account and the total staked
/// amount on the system.
///
/// NOTE: This instruction is permissionless, so anybody can finish
/// deactivating someone's tokens, preparing them to be withdrawn.
///
/// 0. `[w]` Stake config account
/// 1. `[w]` Validator stake account
pub fn process_inactivate_stake(
    program_id: &Pubkey,
    ctx: Context<InactivateStakeAccounts>,
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
    // - must be initialized
    // - must have the correct derivation

    require!(
        ctx.accounts.stake.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "stake"
    );

    let data = &mut ctx.accounts.stake.try_borrow_mut_data()?;
    let stake = bytemuck::try_from_bytes_mut::<Stake>(data)
        .map_err(|_error| ProgramError::InvalidAccountData)?;

    require!(
        stake.is_initialized(),
        ProgramError::UninitializedAccount,
        "stake",
    );

    // validates that the stake account corresponds to the received
    // config account
    let (derivation, _) =
        find_stake_pda(&stake.validator_vote, ctx.accounts.config.key, program_id);

    require!(
        ctx.accounts.stake.key == &derivation,
        ProgramError::InvalidSeeds,
        "stake",
    );

    // Inactivates the stake if elegible.

    if let Some(timestamp) = stake.deactivation_timestamp {
        let inactive_timestamp = config.cooldown_time_seconds.saturating_add(timestamp.get());
        let current_timestamp = Clock::get()?.unix_timestamp as u64;

        require!(
            current_timestamp >= inactive_timestamp,
            StakeError::ActiveDeactivationCooldown,
            "{} second(s) remaining for deactivation",
            inactive_timestamp.saturating_sub(current_timestamp),
        );

        msg!("Inactivating {} token(s)", stake.deactivating_amount);

        // moves deactivating amount to inactive
        stake.amount = stake
            .amount
            .checked_sub(stake.deactivating_amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        stake.inactive_amount = stake
            .inactive_amount
            .checked_add(stake.deactivating_amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        // Update the config and stake account data.

        config.token_amount_delegated = config
            .token_amount_delegated
            .checked_sub(stake.deactivating_amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        // Clears the deactivation.

        stake.deactivating_amount = 0;
        stake.deactivation_timestamp = None;

        Ok(())
    } else {
        Err(StakeError::NoDeactivatedTokens.into())
    }
}
