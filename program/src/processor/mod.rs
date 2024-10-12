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
        find_sol_staker_stake_pda, find_validator_stake_pda, Config, Delegation, SolStakerStake,
        ValidatorStake,
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

pub(crate) fn harvest(
    (config, config_info): (&Config, &AccountInfo),
    delegation: &mut Delegation,
    recipient: &AccountInfo,
    keeper: Option<&AccountInfo>,
) -> ProgramResult {
    // Compute the delegation's accrued rewards.
    let accumulated_rewards_per_token = u128::from(config.accumulated_stake_rewards_per_token);
    let last_seen_stake_rewards_per_token =
        u128::from(delegation.last_seen_stake_rewards_per_token);
    let effective_amount = delegation.effective_amount;
    let total_reward = calculate_eligible_rewards(
        accumulated_rewards_per_token,
        last_seen_stake_rewards_per_token,
        effective_amount,
    )?;

    // Update the last seen amount.
    delegation.last_seen_stake_rewards_per_token = config.accumulated_stake_rewards_per_token;

    // Withdraw the lamports from the config account.
    let rent_exempt_minimum = Rent::get()?.minimum_balance(config_info.data_len());
    let config_lamports = config_info
        .lamports()
        .checked_sub(total_reward)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    assert!(config_lamports >= rent_exempt_minimum);
    **config_info.try_borrow_mut_lamports()? = config_lamports;

    // Pay the keeper first, if one is set.
    let keeper_reward = match keeper {
        Some(keeper) => {
            let keeper_reward = std::cmp::min(total_reward, config.sync_rewards_lamports);
            **keeper.try_borrow_mut_lamports()? = keeper
                .try_borrow_lamports()?
                .checked_add(keeper_reward)
                .ok_or(ProgramError::ArithmeticOverflow)?;

            keeper_reward
        }
        None => 0,
    };

    // Pay the delegator.
    let delegator_reward = total_reward
        .checked_sub(keeper_reward)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    **recipient.try_borrow_mut_lamports()? = recipient
        .lamports()
        .checked_add(delegator_reward)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    Ok(())
}

/// Arguments to process the slash of a stake delegation.
struct SlashArgs<'a, 'b> {
    config: &'b mut Config,
    delegation: &'b mut Delegation,
    lamports_stake: u64,
    mint_info: &'b AccountInfo<'a>,
    vault_info: &'b AccountInfo<'a>,
    vault_authority_info: &'b AccountInfo<'a>,
    token_program_info: &'b AccountInfo<'a>,
    signer_seeds: &'b [&'b [u8]],
    amount: u64,
}

/// Processes the slash for a stake delegation.
fn process_slash_for_delegation(args: SlashArgs) -> ProgramResult {
    let SlashArgs {
        config,
        delegation,
        lamports_stake,
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

    // TODO:
    //
    // - Cap slash at `delegation.amount`.
    // - Perform the slash.
    // - Adjust deactivating amount as necessary.
    // - Update all stake values & global effective stake.

    // Compute actual slash & new stake numbers.
    let actual_slash = std::cmp::min(amount, delegation.amount);
    let total = delegation
        .amount
        .checked_sub(actual_slash)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    let limit = calculate_maximum_stake_for_lamports_amount(lamports_stake)?;
    let effective = std::cmp::min(total, limit);
    let effective_delta = delegation
        .effective_amount
        .checked_sub(effective)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    // Update stake amounts.
    delegation.amount = total;
    delegation.effective_amount = effective;
    config.token_amount_effective = config
        .token_amount_effective
        .checked_sub(effective_delta)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    // Check if we need to downwards adjust deactivating amount.
    if delegation.deactivating_amount > delegation.amount {
        delegation.deactivating_amount = delegation.amount;
        // TODO: Do we need to set deactivation timestamp to none?
    }

    // Burn the tokens from the vault account (if there are tokens to slash).
    if actual_slash > 0 {
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
            actual_slash,
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

    Ok(())
}
