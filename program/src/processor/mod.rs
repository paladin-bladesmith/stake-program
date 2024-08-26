use std::cmp::min;

use bytemuck::Pod;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program::invoke_signed,
    program_error::ProgramError, program_pack::IsInitialized, pubkey::Pubkey, rent::Rent,
    sysvar::Sysvar,
};
use spl_discriminator::{ArrayDiscriminator, SplDiscriminate};
use spl_token_2022::{extension::PodStateWithExtensions, instruction::burn_checked, pod::PodMint};

use crate::{
    error::StakeError,
    instruction::{
        accounts::{
            DeactivateStakeAccounts, DistributeRewardsAccounts, HarvestHolderRewardsAccounts,
            HarvestSolStakerRewardsAccounts, HarvestSyncRewardsAccounts,
            HarvestValidatorRewardsAccounts, InactivateSolStakerStakeAccounts,
            InactivateValidatorStakeAccounts, InitializeConfigAccounts,
            InitializeSolStakerStakeAccounts, InitializeValidatorStakeAccounts,
            SetAuthorityAccounts, SlashSolStakerStakeAccounts, SlashValidatorStakeAccounts,
            SolStakerStakeTokensAccounts, SyncSolStakeAccounts, UpdateConfigAccounts,
            ValidatorStakeTokensAccounts, WithdrawInactiveStakeAccounts,
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
mod harvest_sync_rewards;
mod harvest_validator_rewards;
mod inactivate_sol_staker_stake;
mod inactivate_validator_stake;
mod initialize_config;
mod initialize_sol_staker_stake;
mod initialize_validator_stake;
mod set_authority;
mod slash_sol_staker_stake;
mod slash_validator_stake;
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
        StakeInstruction::InactivateValidatorStake => {
            msg!("Instruction: InactivateValidatorStake");
            inactivate_validator_stake::process_inactivate_validator_stake(
                program_id,
                InactivateValidatorStakeAccounts::context(accounts)?,
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
        StakeInstruction::SlashValidatorStake(amount) => {
            msg!("Instruction: SlashValidatorStake");
            slash_validator_stake::process_slash_validator_stake(
                program_id,
                SlashValidatorStakeAccounts::context(accounts)?,
                amount,
            )
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
        StakeInstruction::InactivateSolStakerStake => {
            msg!("Instruction: InactivateSolStakerStake");
            inactivate_sol_staker_stake::process_inactivate_sol_staker_stake(
                program_id,
                InactivateSolStakerStakeAccounts::context(accounts)?,
            )
        }
        StakeInstruction::SlashSolStakerStake(amount) => {
            msg!("Instruction: SlashSolStakerStake");
            slash_sol_staker_stake::process_slash_sol_staker_stake(
                program_id,
                SlashSolStakerStakeAccounts::context(accounts)?,
                amount,
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
                let accumulated = accumulated_rewards_per_token.wrapping_add(rewards_per_token);
                config.accumulated_stake_rewards_per_token = accumulated.into();
            }
        }

        // Update the last seen stake rewards.
        delegation.last_seen_stake_rewards_per_token = config.accumulated_stake_rewards_per_token;

        Ok(())
    }
}

/// Arguments to process the slash of a stake delegation.
struct SlashArgs<'a, 'b> {
    config: &'b mut Config,
    delegation: &'b mut Delegation,
    mint_info: &'b AccountInfo<'a>,
    vault_info: &'b AccountInfo<'a>,
    vault_authority_info: &'b AccountInfo<'a>,
    token_program_info: &'b AccountInfo<'a>,
    signer_seeds: &'b [&'b [u8]],
    amount: u64,
}

/// Processes the slash for a stake delegation.
fn process_slash_for_delegation(args: SlashArgs) -> Result<u64, ProgramError> {
    let SlashArgs {
        config,
        delegation,
        mint_info,
        vault_info,
        vault_authority_info,
        token_program_info,
        signer_seeds,
        amount,
    } = args;

    // Update the stake amount on both stake and config accounts:
    //
    //   1. the amount slashed is taken from the stake amount (this includes
    //      the amount that is currently staked + deactivating);
    //
    //   2. if not enough, the remaining is ignored and the stake account is
    //      left with 0 amount;
    //
    //   3. if the stake account has deactivating tokens, make sure that the
    //      deactivating amount is at least the same as the stake amount (it might
    //      need to be reduced due to slashing).

    require!(
        amount > 0,
        StakeError::InvalidAmount,
        "amount must be greater than 0"
    );

    // slashes active stake
    let active_stake_to_slash = min(amount, delegation.amount);
    delegation.amount = delegation
        .amount
        .checked_sub(active_stake_to_slash)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    if delegation.deactivating_amount > delegation.amount {
        // adjust the deactivating amount if it's greater than the "active" stake
        // after slashing
        delegation.deactivating_amount = delegation.amount;
        // clear the deactivation timestamp if there in no deactivating
        // amount left
        if delegation.deactivating_amount == 0 {
            delegation.deactivation_timestamp = None;
        }
    }

    // Update the token amount delegated on the config account.
    //
    // The instruction will fail if the amount to slash is greater than the
    // total amount delegated (it should never happen).
    require!(
        config.token_amount_delegated >= active_stake_to_slash,
        StakeError::InvalidSlashAmount,
        "slash amount greater than total amount delegated ({} required, {} delegated)",
        active_stake_to_slash,
        config.token_amount_delegated
    );

    config.token_amount_delegated = config
        .token_amount_delegated
        .checked_sub(active_stake_to_slash)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    if amount > active_stake_to_slash {
        // The amount to slash was greater than the amount available on the
        // stake account.
        msg!(
            "Slash amount greater than available tokens on stake account ({} required, {} available)",
            amount,
            active_stake_to_slash,
        );
    }

    // Burn the tokens from the vault account (if there are tokens to slash).

    if active_stake_to_slash > 0 {
        let mint_data = mint_info.try_borrow_data()?;
        let mint = PodStateWithExtensions::<PodMint>::unpack(&mint_data)?;
        let decimals = mint.base.decimals;

        drop(mint_data);

        let burn_ix = burn_checked(
            token_program_info.key,
            vault_info.key,
            mint_info.key,
            vault_authority_info.key,
            &[],
            active_stake_to_slash,
            decimals,
        )?;

        invoke_signed(
            &burn_ix,
            &[
                token_program_info.clone(),
                vault_info.clone(),
                mint_info.clone(),
                vault_authority_info.clone(),
            ],
            &[signer_seeds],
        )?;
    }

    Ok(active_stake_to_slash)
}
