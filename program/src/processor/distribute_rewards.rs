use solana_program::{
    entrypoint::ProgramResult, program::invoke, program_error::ProgramError, pubkey::Pubkey,
    system_instruction, system_program,
};

use crate::{
    instruction::accounts::{Context, DistributeRewardsAccounts},
    require,
    state::{calculate_stake_rewards_per_token, Config},
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
    // Account validation.

    // config
    // - owner must be the stake program
    // - must be initialized

    require!(
        ctx.accounts.config.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "config"
    );

    let mut config_data = ctx.accounts.config.try_borrow_mut_data()?;
    let config = bytemuck::try_from_bytes_mut::<Config>(&mut config_data)
        .map_err(|_error| ProgramError::InvalidAccountData)?;

    require!(
        config.is_initialized(),
        ProgramError::UninitializedAccount,
        "config",
    );

    // payer
    // - must be a signer

    require!(
        ctx.accounts.payer.is_signer,
        ProgramError::MissingRequiredSignature,
        "payer",
    );

    // system program
    // - must be the system program

    require!(
        ctx.accounts.system_program.key == &system_program::ID,
        ProgramError::IncorrectProgramId,
        "system program"
    );

    // Calculates the rewards per token

    let rewards_per_token =
        calculate_stake_rewards_per_token(amount, config.token_amount_delegated)?;

    if rewards_per_token != 0 {
        // updates the accumulated stake rewards per token
        let accumulated = config
            .accumulated_stake_rewards_per_token()
            .checked_add(rewards_per_token)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        config.set_accumulated_stake_rewards_per_token(accumulated);
    }

    // Transfers the rewards to the config account.

    drop(config_data);

    invoke(
        &system_instruction::transfer(ctx.accounts.payer.key, ctx.accounts.config.key, amount),
        &[ctx.accounts.payer.clone(), ctx.accounts.config.clone()],
    )?;

    Ok(())
}
