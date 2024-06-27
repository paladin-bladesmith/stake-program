use solana_program::{
    entrypoint::ProgramResult, program::invoke, program_error::ProgramError, pubkey::Pubkey,
    system_instruction,
};

use crate::{
    instruction::accounts::{Context, DistributeRewardsAccounts},
    require,
    state::Config,
};

/// Moves SOL rewards to the config and updates the stake rewards total
///
/// Accounts expected by this instruction:
///
/// 0. `[w,s]` Reward payer
/// 1. `[w]` Config account
/// 2. `[]` System Program
pub fn process_distribute_rewards(
    program_id: &Pubkey,
    ctx: Context<DistributeRewardsAccounts>,
    amount: u64,
) -> ProgramResult {
    // Accounts valudation.

    // 1. config
    // - owner must the stake program
    // - must be initialized

    require!(
        ctx.accounts.config.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "config"
    );

    let mut data = ctx.accounts.config.try_borrow_mut_data()?;

    let config = bytemuck::try_from_bytes_mut::<Config>(&mut data)
        .map_err(|_error| ProgramError::InvalidAccountData)?;

    require!(
        config.is_initialized(),
        ProgramError::UninitializedAccount,
        "config"
    );

    // updates the stake rewards total on the config

    config.total_stake_rewards = config.total_stake_rewards.saturating_add(amount);
    drop(data);

    // transfer the rewards to the config

    invoke(
        &system_instruction::transfer(ctx.accounts.payer.key, ctx.accounts.config.key, amount),
        &[
            ctx.accounts.payer.clone(),
            ctx.accounts.config.clone(),
            ctx.accounts.system_program.clone(),
        ],
    )
}
