use bytemuck::Pod;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    program_pack::IsInitialized, pubkey::Pubkey,
};
use spl_discriminator::{ArrayDiscriminator, SplDiscriminate};

use crate::{
    instruction::{
        accounts::{
            DeactivateStakeAccounts, DistributeRewardsAccounts, HarvestHolderRewardsAccounts,
            InactivateStakeAccounts, InitializeConfigAccounts, InitializeSolStakerStakeAccounts,
            InitializeValidatorStakeAccounts, SetAuthorityAccounts, UpdateConfigAccounts,
            WithdrawInactiveStakeAccounts,
        },
        StakeInstruction,
    },
    state::{
        find_sol_staker_stake_pda, find_validator_stake_pda, Delegation, SolStakerStake,
        ValidatorStake,
    },
};

mod deactivate_stake;
mod distribute_rewards;
mod harvest_holder_rewards;
//mod harvest_stake_rewards;
mod inactivate_stake;
mod initialize_config;
mod initialize_sol_staker_stake;
mod initialize_validator_stake;
mod set_authority;
//mod slash;
//mod stake_tokens;
mod update_config;
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
        StakeInstruction::HarvestStakeRewards => {
            msg!("Instruction: HarvestStakeRewards");
            // Note: needs to be refactored
            //harvest_stake_rewards::process_harvest_stake_rewards(
            //    program_id,
            //    HarvestStakeRewardsAccounts::context(accounts)?,
            //)
            todo!();
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
        } => {
            msg!("Instruction: InitializeConfig");
            initialize_config::process_initialize_config(
                program_id,
                InitializeConfigAccounts::context(accounts)?,
                cooldown_time_seconds,
                max_deactivation_basis_points,
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
        StakeInstruction::StakeTokens(_amount) => {
            msg!("Instruction: StakeTokens");
            // Note: needs to be refactored
            //stake_tokens::process_stake_tokens(
            //    program_id,
            //    StakeTokensAccounts::context(accounts)?,
            //    amount,
            //)
            todo!();
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
/// This function will validate that the account data is initialized and deriavation matches
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
                find_sol_staker_stake_pda(&sol_staker.stake_state, config, program_id);

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
