use borsh::BorshDeserialize;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::instruction::{
    accounts::{
        DeactivateStakeAccounts, DistributeRewardsAccounts, HarvestHolderRewardsAccounts,
        HarvestStakeRewardsAccounts, InactivateStakeAccounts, InitializeConfigAccounts,
        InitializeStakeAccounts, SetAuthorityAccounts, SlashAccounts, StakeTokensAccounts,
        UpdateConfigAccounts, WithdrawInactiveStakeAccounts,
    },
    Instruction,
};

mod deactivate_stake;
mod distribute_rewards;
mod harvest_holder_rewards;
mod harvest_stake_rewards;
mod inactivate_stake;
mod initialize_config;
mod initialize_stake;
mod set_authority;
mod slash;
mod stake_tokens;
mod update_config;
mod withdraw_inactive_stake;

#[inline(always)]
pub fn process_instruction<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &[u8],
) -> ProgramResult {
    // TODO: should use Pod types for instruction data?
    let instruction: Instruction = Instruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        Instruction::DeactivateStake(amount) => {
            msg!("Instruction: DeactivateStake");
            deactivate_stake::process_deactivate_stake(
                program_id,
                DeactivateStakeAccounts::context(accounts)?,
                amount,
            )
        }
        Instruction::DistributeRewards(amount) => {
            msg!("Instruction: DistributeRewards");
            distribute_rewards::process_distribute_rewards(
                program_id,
                DistributeRewardsAccounts::context(accounts)?,
                amount,
            )
        }
        Instruction::HarvestHolderRewards => {
            msg!("Instruction: HarvestHolderRewards");
            harvest_holder_rewards::process_harvest_holder_rewards(
                program_id,
                HarvestHolderRewardsAccounts::context(accounts)?,
            )
        }
        Instruction::HarvestStakeRewards => {
            msg!("Instruction: HarvestStakeRewards");
            harvest_stake_rewards::process_harvest_stake_rewards(
                program_id,
                HarvestStakeRewardsAccounts::context(accounts)?,
            )
        }
        Instruction::InactivateStake => {
            msg!("Instruction: InactivateStake");
            inactivate_stake::process_inactivate_stake(
                program_id,
                InactivateStakeAccounts::context(accounts)?,
            )
        }
        Instruction::InitializeConfig {
            cooldown_time,
            max_deactivation_basis_points,
        } => {
            msg!("Instruction: InitializeConfig");
            initialize_config::process_initialize_config(
                program_id,
                InitializeConfigAccounts::context(accounts)?,
                cooldown_time,
                max_deactivation_basis_points,
            )
        }
        Instruction::InitializeStake => {
            msg!("Instruction: InitializeStake");
            initialize_stake::process_initialize_stake(
                program_id,
                InitializeStakeAccounts::context(accounts)?,
            )
        }
        Instruction::SetAuthority(authority) => {
            msg!("Instruction: SetAuthority");
            set_authority::process_set_authority(
                program_id,
                SetAuthorityAccounts::context(accounts)?,
                authority,
            )
        }
        Instruction::Slash(amount) => {
            msg!("Instruction: Slash");
            slash::process_slash(program_id, SlashAccounts::context(accounts)?, amount)
        }
        Instruction::StakeTokens(amount) => {
            msg!("Instruction: StakeTokens");
            stake_tokens::process_stake_tokens(
                program_id,
                StakeTokensAccounts::context(accounts)?,
                amount,
            )
        }
        Instruction::UpdateConfig(field) => {
            msg!("Instruction: UpdateConfig");
            update_config::process_update_config(
                program_id,
                UpdateConfigAccounts::context(accounts)?,
                field,
            )
        }
        Instruction::WithdrawInactiveStake(amount) => {
            msg!("Instruction: WithdrawInactiveStake");
            withdraw_inactive_stake::process_withdraw_inactive_stake(
                program_id,
                WithdrawInactiveStakeAccounts::context(accounts)?,
                amount,
            )
        }
    }
}

#[macro_export]
macro_rules! require {
    ( $constraint:expr, $error:expr ) => {
        if !$constraint {
            return Err($error.into());
        }
    };
    ( $constraint:expr, $error:expr, $message:expr ) => {
        if !$constraint {
            solana_program::msg!("Constraint failed: {}", $message);
            return Err($error.into());
        }
    };
    ( $constraint:expr, $error:expr, $message:literal, $($args:tt)+ ) => {
        require!( $constraint, $error, format!($message, $($args)+) );
    };
}
