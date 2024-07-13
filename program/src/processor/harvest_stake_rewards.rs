use solana_program::{
    entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey, rent::Rent,
    sysvar::Sysvar,
};

use crate::{
    error::StakeError,
    instruction::accounts::{Context, HarvestStakeRewardsAccounts},
    require,
    state::{calculate_eligible_rewards, find_stake_pda, Config, Stake},
};

/// Harvests stake SOL rewards earned by the given stake account.
///
/// NOTE: This is very similar to the logic in the rewards program. Since the
/// staking rewards are held in a separate account, they must be distributed
/// based on the proportion of total stake.
///
/// 0. `[w]` Config account
/// 1. `[w]` Stake account
/// 2. `[w]` Destination account
/// 3. `[s]` Stake authority
pub fn process_harvest_stake_rewards(
    program_id: &Pubkey,
    ctx: Context<HarvestStakeRewardsAccounts>,
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

    // stake
    // - owner must be the stake program
    // - must be initialized
    // - derivation must match (validates the config account)

    require!(
        ctx.accounts.stake.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "stake"
    );

    let mut stake_data = ctx.accounts.stake.try_borrow_mut_data()?;
    let stake = bytemuck::try_from_bytes_mut::<Stake>(&mut stake_data)
        .map_err(|_error| ProgramError::InvalidAccountData)?;

    require!(
        stake.is_initialized(),
        ProgramError::UninitializedAccount,
        "stake",
    );

    let (derivation, _) = find_stake_pda(&stake.validator, ctx.accounts.config.key, program_id);

    require!(
        ctx.accounts.stake.key == &derivation,
        ProgramError::InvalidSeeds,
        "stake",
    );

    // stake authority
    // - must be a signer
    // - must match the authority on the stake account

    require!(
        ctx.accounts.stake_authority.is_signer,
        ProgramError::MissingRequiredSignature,
        "stake authority",
    );

    require!(
        ctx.accounts.stake_authority.key == &stake.authority,
        StakeError::InvalidAuthority,
        "stake authority",
    );

    // Determine the stake rewards.

    let accumulated_rewards_per_token = config.accumulated_stake_rewards_per_token();
    let rewards = calculate_eligible_rewards(
        accumulated_rewards_per_token,
        stake.last_seen_stake_rewards_per_token(),
        stake.amount,
    )?;
    // update the last seen holder rewards
    stake.set_last_seen_stake_rewards_per_token(accumulated_rewards_per_token);

    // Transfer the rewards to the destination account.

    if rewards != 0 {
        // If the config does not have enough lamports to cover the rewards, only
        // harvest the available lamports. This should never happen, but the check
        // is a failsafe.
        let rewards = {
            let rent = Rent::get()?;
            let rent_exempt_lamports = rent.minimum_balance(Config::LEN);

            std::cmp::min(
                rewards,
                ctx.accounts
                    .config
                    .lamports()
                    .saturating_sub(rent_exempt_lamports),
            )
        };

        // Move the amount from the config to the destination account.
        let updated_config_lamports = ctx
            .accounts
            .config
            .lamports()
            .checked_sub(rewards)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        let updated_destination_lamports = ctx
            .accounts
            .destination
            .lamports()
            .checked_add(rewards)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        **ctx.accounts.config.try_borrow_mut_lamports()? = updated_config_lamports;
        **ctx.accounts.destination.try_borrow_mut_lamports()? = updated_destination_lamports;
    }

    Ok(())
}
