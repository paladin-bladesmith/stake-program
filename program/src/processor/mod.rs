use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey, pubkey::Pubkey,
};

use crate::instruction::{
    accounts::{
        DeactivateStakeAccounts, DistributeRewardsAccounts, HarvestHolderRewardsAccounts,
        HarvestStakeRewardsAccounts, InactivateStakeAccounts, InitializeConfigAccounts,
        InitializeStakeAccounts, SetAuthorityAccounts, SlashAccounts, StakeTokensAccounts,
        UpdateConfigAccounts, WithdrawInactiveStakeAccounts,
    },
    StakeInstruction,
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

// TODO: Replace this with the actual Rewards program ID
const REWARDS_PROGRAM_ID: Pubkey = pubkey!("PStake1111111111111111111111111111111111111");

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
            harvest_stake_rewards::process_harvest_stake_rewards(
                program_id,
                HarvestStakeRewardsAccounts::context(accounts)?,
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
        } => {
            msg!("Instruction: InitializeConfig");
            initialize_config::process_initialize_config(
                program_id,
                InitializeConfigAccounts::context(accounts)?,
                cooldown_time_seconds,
                max_deactivation_basis_points,
            )
        }
        StakeInstruction::InitializeStake => {
            msg!("Instruction: InitializeStake");
            initialize_stake::process_initialize_stake(
                program_id,
                InitializeStakeAccounts::context(accounts)?,
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
        StakeInstruction::Slash(amount) => {
            msg!("Instruction: Slash");
            slash::process_slash(program_id, SlashAccounts::context(accounts)?, amount)
        }
        StakeInstruction::StakeTokens(amount) => {
            msg!("Instruction: StakeTokens");
            stake_tokens::process_stake_tokens(
                program_id,
                StakeTokensAccounts::context(accounts)?,
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
