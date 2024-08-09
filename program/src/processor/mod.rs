use std::cmp::min;

use bytemuck::Pod;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    program_pack::IsInitialized, pubkey::Pubkey, rent::Rent, sysvar::Sysvar,
};
use spl_discriminator::{ArrayDiscriminator, SplDiscriminate};

use crate::{
    instruction::{
        accounts::{
            DeactivateStakeAccounts, DistributeRewardsAccounts, HarvestHolderRewardsAccounts,
            HarvestSolStakerRewardsAccounts, HarvestSyncRewardsAccounts,
            HarvestValidatorRewardsAccounts, InactivateStakeAccounts, InitializeConfigAccounts,
            InitializeSolStakerStakeAccounts, InitializeValidatorStakeAccounts,
            SetAuthorityAccounts, SolStakerStakeTokensAccounts, SyncSolStakeAccounts,
            UpdateConfigAccounts, ValidatorStakeTokensAccounts, WithdrawInactiveStakeAccounts,
        },
        StakeInstruction,
    },
    state::{
        calculate_eligible_rewards, calculate_maximum_stake_for_lamports_amount,
        calculate_stake_rewards_per_token, find_sol_staker_stake_pda, find_validator_stake_pda,
        Config, Delegation, SolStakerStake, ValidatorStake,
    },
};

mod deactivate_stake;
mod distribute_rewards;
mod harvest_holder_rewards;
mod harvest_sol_staker_rewards;
mod harvest_validator_rewards;
mod inactivate_stake;
mod initialize_config;
mod initialize_sol_staker_stake;
mod initialize_validator_stake;
mod set_authority;
//mod slash;
mod harvest_sync_rewards;
mod sol_staker_stake_tokens;
mod sync_sol_stake;
mod update_config;
mod validator_stake_tokens;
mod withdraw_inactive_stake;

#[inline(always)]
pub fn process_instruction<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = StakeInstruction::unpack(instruction_data)?;

    match instruction {
        StakeInstruction::DeactivateStake(amount) => {
            msg!("Instruction: DeactivateStake");
            deactivate_stake::process_deactivate_stake(
                program_id,
                DeactivateStakeAccounts::context(accounts)?,
                amount,
            )
        }
        StakeInstruction::DistributeRewards(amount) => {
            msg!("Instruction: DistributeRewards");
            distribute_rewards::process_distribute_rewards(
                program_id,
                DistributeRewardsAccounts::context(accounts)?,
                amount,
            )
        }
        StakeInstruction::HarvestHolderRewards => {
            msg!("Instruction: HarvestHolderRewards");
            harvest_holder_rewards::process_harvest_holder_rewards(
                program_id,
                HarvestHolderRewardsAccounts::context(accounts)?,
            )
        }
        StakeInstruction::HarvestValidatorRewards => {
            msg!("Instruction: HarvestValidatorRewards");
            harvest_validator_rewards::process_harvest_validator_rewards(
                program_id,
                HarvestValidatorRewardsAccounts::context(accounts)?,
            )
        }
        StakeInstruction::InactivateStake => {
            msg!("Instruction: InactivateStake");
            inactivate_stake::process_inactivate_stake(
                program_id,
                InactivateStakeAccounts::context(accounts)?,
            )
        }
        StakeInstruction::InitializeConfig {
            cooldown_time_seconds,
            max_deactivation_basis_points,
            sync_rewards_lamports,
        } => {
            msg!("Instruction: InitializeConfig");
            initialize_config::process_initialize_config(
                program_id,
                InitializeConfigAccounts::context(accounts)?,
                cooldown_time_seconds,
                max_deactivation_basis_points,
                sync_rewards_lamports,
            )
        }
        StakeInstruction::InitializeValidatorStake => {
            msg!("Instruction: InitializeValidatorStake");
            initialize_validator_stake::process_initialize_validator_stake(
                program_id,
                InitializeValidatorStakeAccounts::context(accounts)?,
            )
        }
        StakeInstruction::SetAuthority(authority) => {
            msg!("Instruction: SetAuthority");
            set_authority::process_set_authority(
                program_id,
                SetAuthorityAccounts::context(accounts)?,
                authority,
            )
        }
        StakeInstruction::Slash(_amount) => {
            msg!("Instruction: Slash");
            // Note: needs to be refactored
            //slash::process_slash(program_id, SlashAccounts::context(accounts)?, amount)
            todo!();
        }
        StakeInstruction::ValidatorStakeTokens(amount) => {
            msg!("Instruction: ValidatorStakeTokens");
            validator_stake_tokens::process_validator_stake_tokens(
                program_id,
                ValidatorStakeTokensAccounts::context(accounts)?,
                amount,
            )
        }
        StakeInstruction::UpdateConfig(field) => {
            msg!("Instruction: UpdateConfig");
            update_config::process_update_config(
                program_id,
                UpdateConfigAccounts::context(accounts)?,
                field,
            )
        }
        StakeInstruction::WithdrawInactiveStake(amount) => {
            msg!("Instruction: WithdrawInactiveStake");
            withdraw_inactive_stake::process_withdraw_inactive_stake(
                program_id,
                WithdrawInactiveStakeAccounts::context(accounts)?,
                amount,
            )
        }
        StakeInstruction::InitializeSolStakerStake => {
            msg!("Instruction: InitializeSolStakerStake");
            initialize_sol_staker_stake::process_initialize_sol_staker_stake(
                program_id,
                InitializeSolStakerStakeAccounts::context(accounts)?,
            )
        }
        StakeInstruction::SolStakerStakeTokens(amount) => {
            msg!("Instruction: SolStakerStakeTokens");
            sol_staker_stake_tokens::process_sol_staker_stake_tokens(
                program_id,
                SolStakerStakeTokensAccounts::context(accounts)?,
                amount,
            )
        }
        StakeInstruction::SyncSolStake => {
            msg!("Instruction: SyncSolStake");
            sync_sol_stake::process_sync_sol_stake(
                program_id,
                SyncSolStakeAccounts::context(accounts)?,
            )
        }
        StakeInstruction::HarvestSolStakerRewards => {
            msg!("Instruction: HarvestSolStakerRewards");
            harvest_sol_staker_rewards::process_harvest_sol_staker_rewards(
                program_id,
                HarvestSolStakerRewardsAccounts::context(accounts)?,
            )
        }
        StakeInstruction::HarvestSyncRewards => {
            msg!("Instruction: HarvestSyncRewards");
            harvest_sync_rewards::process_harvest_sync_rewards(
                program_id,
                HarvestSyncRewardsAccounts::context(accounts)?,
            )
        }
    }
}

#[macro_export]
macro_rules! require {
    ( $constraint:expr, $error:expr $(,)? ) => {
        if !$constraint {
            return Err($error.into());
        }
    };
    ( $constraint:expr, $error:expr, $message:expr $(,)? ) => {
        if !$constraint {
            solana_program::msg!("Constraint failed: {}", $message);
            return Err($error.into());
        }
    };
    ( $constraint:expr, $error:expr, $message:literal, $($args:tt)+ ) => {
        require!( $constraint, $error, format!($message, $($args)+) );
    };
}

/// Unpacks an initialized account from the given data and
/// returns a mutable reference to it.
#[inline]
pub fn unpack_initialized_mut<T: Pod + IsInitialized>(
    data: &mut [u8],
) -> Result<&mut T, ProgramError> {
    let account = bytemuck::try_from_bytes_mut::<T>(data)
        .map_err(|_error| ProgramError::InvalidAccountData)?;

    require!(account.is_initialized(), ProgramError::UninitializedAccount);

    Ok(account)
}

/// Unpacks the delegation information from either a `SolStakerStake` and `ValidatorStake`
/// accounts.
///
/// This function will validate that the account data is initialized and derivation matches
/// the expected PDA derivation.
#[inline]
pub fn unpack_delegation_mut<'a>(
    stake_data: &'a mut [u8],
    stake: &Pubkey,
    config: &Pubkey,
    program_id: &Pubkey,
) -> Result<&'a mut Delegation, ProgramError> {
    let (delegation, derivation) = match &stake_data[..ArrayDiscriminator::LENGTH] {
        SolStakerStake::SPL_DISCRIMINATOR_SLICE => {
            let sol_staker = unpack_initialized_mut::<SolStakerStake>(stake_data)?;

            let (derivation, _) =
                find_sol_staker_stake_pda(&sol_staker.sol_stake, config, program_id);

            (&mut sol_staker.delegation, derivation)
        }
        ValidatorStake::SPL_DISCRIMINATOR_SLICE => {
            let validator = unpack_initialized_mut::<ValidatorStake>(stake_data)?;

            let (derivation, _) =
                find_validator_stake_pda(&validator.delegation.validator_vote, config, program_id);

            (&mut validator.delegation, derivation)
        }
        _ => return Err(ProgramError::InvalidAccountData),
    };

    require!(stake == &derivation, ProgramError::InvalidSeeds, "stake");

    Ok(delegation)
}

/// Unpacks the delegation information from either a `SolStakerStake` and `ValidatorStake`
/// accounts.
///
/// This function will validate that the account data is initialized.
#[inline]
pub fn unpack_delegation_mut_uncheked(
    stake_data: &mut [u8],
) -> Result<&mut Delegation, ProgramError> {
    let delegation = match &stake_data[..ArrayDiscriminator::LENGTH] {
        SolStakerStake::SPL_DISCRIMINATOR_SLICE => {
            let sol_staker = unpack_initialized_mut::<SolStakerStake>(stake_data)?;
            &mut sol_staker.delegation
        }
        ValidatorStake::SPL_DISCRIMINATOR_SLICE => {
            let validator = unpack_initialized_mut::<ValidatorStake>(stake_data)?;
            &mut validator.delegation
        }
        _ => return Err(ProgramError::InvalidAccountData),
    };

    Ok(delegation)
}

/// Processes the harvest for a stake delegation.
///
/// This function will calculate the rewards for the delegation, either `ValidatorStake` or
/// `SolStakerStake` and transfer the elegible rewards. It will check whether the stake amount
/// exceeds the maximum stake limit and distribute the rewards back to the stakers if the stake
/// amount exceeds the limit.
pub fn process_harvest_for_delegation(
    config: &mut Config,
    delegation: &mut Delegation,
    lamports_amount: u64,
    config_info: &AccountInfo,
    destination_info: &AccountInfo,
) -> ProgramResult {
    // Determine the stake rewards.
    //
    // The rewards are caped using the minimum of the token amount staked and the
    // token amount limit calculated from the SOL amount staked. This is to prevent getting
    // rewards from exceeding token amount based on the SOL amount staked.
    //
    // Any excess rewards are distributes back to stakers and the SOL staker forfeits
    // the rewards for the exceeding amount.

    let stake_limit = calculate_maximum_stake_for_lamports_amount(lamports_amount)?;
    let validated_amount = min(delegation.amount, stake_limit as u64);

    let accumulated_rewards_per_token = u128::from(config.accumulated_stake_rewards_per_token);
    let last_seen_stake_rewards_per_token =
        u128::from(delegation.last_seen_stake_rewards_per_token);

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
                config_info.lamports().saturating_sub(rent_exempt_lamports),
            )
        };

        // Move the amount from the config to the destination account.
        let updated_config_lamports = config_info
            .lamports()
            .checked_sub(rewards)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        let updated_destination_lamports = destination_info
            .lamports()
            .checked_add(rewards)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        **config_info.try_borrow_mut_lamports()? = updated_config_lamports;
        **destination_info.try_borrow_mut_lamports()? = updated_destination_lamports;

        // Check whether the staked token amount exceeded the validated amount.
        //
        // When there is an excess, the rewards for the excess token amount are distributed back
        // to the stakers without taking into consideration the stakers' stake amount – i.e.,
        // the rewards for the exceeding amount are forfeited by the staker.
        if validated_amount < delegation.amount {
            msg!(
                "Staked amount ({}) exceeds maximum stake limit ({})",
                delegation.amount,
                stake_limit
            );

            let excess = delegation
                .amount
                .checked_sub(stake_limit)
                .ok_or(ProgramError::ArithmeticOverflow)?;

            let excess_rewards = calculate_eligible_rewards(
                accumulated_rewards_per_token,
                last_seen_stake_rewards_per_token,
                excess as u64,
            )?;

            // Calculate the rewards per token without considering the stakes' stake
            // amount, since the staker with the exceeding amount won't be claiming a
            // a share of these rewards.
            let rewards_per_token = calculate_stake_rewards_per_token(
                excess_rewards,
                config
                    .token_amount_delegated
                    .checked_sub(delegation.amount)
                    .ok_or(ProgramError::ArithmeticOverflow)?,
            )?;

            if rewards_per_token != 0 {
                // Update the accumulated stake rewards per token on the config to
                // reflect the addition of the rewards for the exceeding amount.
                let accumulated = accumulated_rewards_per_token
                    .checked_add(rewards_per_token)
                    .ok_or(ProgramError::ArithmeticOverflow)?;
                config.accumulated_stake_rewards_per_token = accumulated.into();
            }
        }

        // Update the last seen stake rewards.
        delegation.last_seen_stake_rewards_per_token = config.accumulated_stake_rewards_per_token;

        Ok(())
    }
}
