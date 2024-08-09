use std::cmp::min;

use solana_program::{
    entrypoint::ProgramResult, msg, program_error::ProgramError, pubkey::Pubkey, rent::Rent,
    sysvar::Sysvar,
};

use crate::{
    error::StakeError,
    instruction::accounts::{Context, HarvestValidatorRewardsAccounts},
    processor::unpack_initialized_mut,
    require,
    state::{
        calculate_eligible_rewards, calculate_maximum_stake_for_lamports_amount,
        calculate_stake_rewards_per_token, find_validator_stake_pda, Config, ValidatorStake,
    },
};

/// Harvests stake SOL rewards earned by the given stake account.
///
/// NOTE: This is very similar to the logic in the rewards program. Since the
/// staking rewards are held in a separate account, they must be distributed
/// based on the proportion of total stake.
///
/// 0. `[w]` Config account
/// 1. `[w]` Validator stake account
/// 2. `[w]` Destination account
/// 3. `[s]` Stake authority
pub fn process_harvest_validator_rewards(
    program_id: &Pubkey,
    ctx: Context<HarvestValidatorRewardsAccounts>,
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
        ctx.accounts.validator_stake.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "validator stake"
    );

    let mut stake_data = ctx.accounts.validator_stake.try_borrow_mut_data()?;
    let validator_stake = unpack_initialized_mut::<ValidatorStake>(&mut stake_data)?;

    let (derivation, _) = find_validator_stake_pda(
        &validator_stake.delegation.validator_vote,
        ctx.accounts.config.key,
        program_id,
    );

    require!(
        ctx.accounts.validator_stake.key == &derivation,
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
        ctx.accounts.stake_authority.key == &validator_stake.delegation.authority,
        StakeError::InvalidAuthority,
        "stake authority",
    );

    // Determine the stake rewards.
    //
    // The rewards are caped using the minimum of the total amount staked and the
    // limit calculated from the total SOL amount staked. This is to prevent getting
    // rewards from exceeding token amount based on the total SOL amount staked.
    //
    // Any excess rewards are distributes back to stakers and the validator forfeits
    // its share of them.

    let stake_limit =
        calculate_maximum_stake_for_lamports_amount(validator_stake.total_staked_lamports_amount)?;
    let validated_amount = min(validator_stake.delegation.amount, stake_limit as u64);

    let accumulated_rewards_per_token = u128::from(config.accumulated_stake_rewards_per_token);
    let last_seen_stake_rewards_per_token =
        u128::from(validator_stake.delegation.last_seen_stake_rewards_per_token);

    let rewards = calculate_eligible_rewards(
        accumulated_rewards_per_token,
        last_seen_stake_rewards_per_token,
        validated_amount,
    )?;

    // Transfer the rewards to the destination account.

    if rewards == 0 {
        msg!("No rewards to harvest");
        Ok(())
    } else {
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

        // Check whether the staked amount exceeded the validated amount.
        //
        // When there is an excess, the rewards for the excess token amount are distributed back
        // to the stakers without taking into consideration the validator's stake amount – i.e.,
        // the rewards for the exceeding amount are forfeited by the validator.
        if validated_amount < validator_stake.delegation.amount {
            msg!(
                "Staked amount ({}) exceeds maximum stake limit ({})",
                validator_stake.delegation.amount,
                stake_limit
            );

            let excess = (validator_stake.delegation.amount)
                .checked_sub(stake_limit)
                .ok_or(ProgramError::ArithmeticOverflow)?;

            let excess_rewards = calculate_eligible_rewards(
                accumulated_rewards_per_token,
                last_seen_stake_rewards_per_token,
                excess as u64,
            )?;

            // Calculates the rewards per token without considering the validators' stake
            // amount, since the validator with the exceeding amount won't be claiming a
            // a share of these rewards.
            let rewards_per_token = calculate_stake_rewards_per_token(
                excess_rewards,
                config
                    .token_amount_delegated
                    .checked_sub(validator_stake.delegation.amount)
                    .ok_or(ProgramError::ArithmeticOverflow)?,
            )?;

            if rewards_per_token != 0 {
                // Updates the accumulated stake rewards per token on the config to
                // reflect the addition of the rewards for the exceeding amount.
                let accumulated = accumulated_rewards_per_token
                    .checked_add(rewards_per_token)
                    .ok_or(ProgramError::ArithmeticOverflow)?;
                config.accumulated_stake_rewards_per_token = accumulated.into();
            }
        }

        // update the last seen stake rewards
        validator_stake.delegation.last_seen_stake_rewards_per_token =
            config.accumulated_stake_rewards_per_token;

        Ok(())
    }
}
