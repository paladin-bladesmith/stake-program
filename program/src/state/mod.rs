pub mod config;
pub mod sol_staker_stake;
pub mod validator_stake;

pub use config::*;
pub use sol_staker_stake::*;
use spl_pod::primitives::PodU128;
pub use validator_stake::*;

use bytemuck::{Pod, Zeroable};
use shank::ShankType;
use solana_program::{
    program_error::ProgramError,
    pubkey::{Pubkey, PubkeyError},
};
use std::num::NonZeroU64;

/// Scaling factor for rewards per token (1e9).
const REWARDS_PER_TOKEN_SCALING_FACTOR: u128 = 1_000_000_000;

/// Defined the maximum value for basis points (100%).
pub const MAX_BASIS_POINTS: u128 = 10_000;

/// Stake factor for the maximum amount of staked tokens as a percentage of the total
/// SOL staked (STAKE_FACTOR / STAKE_SCALING_FACTOR).
pub const STAKE_FACTOR: u128 = 13;

/// Scaling factor for stake amount.
pub const STAKE_SCALING_FACTOR: u128 = 10;

#[inline(always)]
pub fn create_vault_pda<'a>(
    config: &'a Pubkey,
    bump_seed: &'a [u8],
    program_id: &'a Pubkey,
) -> Result<Pubkey, PubkeyError> {
    Pubkey::create_program_address(&get_vault_pda_signer_seeds(config, bump_seed), program_id)
}

#[inline(always)]
pub fn find_vault_pda(config: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&["token-owner".as_bytes(), config.as_ref()], program_id)
}

#[inline(always)]
pub fn find_sol_staker_stake_pda(
    stake_state: &Pubkey,
    config: &Pubkey,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            "stake::state::sol_staker_stake".as_bytes(),
            stake_state.as_ref(),
            config.as_ref(),
        ],
        program_id,
    )
}

#[inline(always)]
pub fn find_validator_stake_pda(
    validator_vote: &Pubkey,
    config: &Pubkey,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            "stake::state::validator_stake".as_bytes(),
            validator_vote.as_ref(),
            config.as_ref(),
        ],
        program_id,
    )
}

#[inline(always)]
pub fn get_vault_pda_signer_seeds<'a>(config: &'a Pubkey, bump_seed: &'a [u8]) -> [&'a [u8]; 3] {
    ["token-owner".as_bytes(), config.as_ref(), bump_seed]
}

#[inline(always)]
pub fn get_sol_staker_stake_pda_signer_seeds<'a>(
    stake_state: &'a Pubkey,
    config: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 4] {
    [
        "stake::state::sol_staker_stake".as_bytes(),
        stake_state.as_ref(),
        config.as_ref(),
        bump_seed,
    ]
}

#[inline(always)]
pub fn get_validator_stake_pda_signer_seeds<'a>(
    validator_vote: &'a Pubkey,
    config: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 4] {
    [
        "stake::state::validator_stake".as_bytes(),
        validator_vote.as_ref(),
        config.as_ref(),
        bump_seed,
    ]
}

/// Calculate the eligible rewards for a given token account balance.
///
/// This uses the same logic as the Rewards program.
pub fn calculate_eligible_rewards(
    current_accumulated_rewards_per_token: u128,
    last_accumulated_rewards_per_token: u128,
    token_account_balance: u64,
) -> Result<u64, ProgramError> {
    // Calculation:
    //   (current_accumulated_rewards_per_token
    //     - last_accumulated_rewards_per_token)
    //   * token_account_balance
    let marginal_rate = current_accumulated_rewards_per_token
        .checked_sub(last_accumulated_rewards_per_token)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    if marginal_rate == 0 {
        Ok(0)
    } else {
        // Scaled by 1e9 to store 9 decimal places of precision.
        marginal_rate
            .checked_mul(token_account_balance as u128)
            .and_then(|product| product.checked_div(REWARDS_PER_TOKEN_SCALING_FACTOR))
            .and_then(|product| product.try_into().ok())
            .ok_or(ProgramError::ArithmeticOverflow)
    }
}

pub fn calculate_stake_rewards_per_token(
    rewards: u64,
    stake_amount: u64,
) -> Result<u128, ProgramError> {
    if stake_amount == 0 {
        Ok(0)
    } else {
        // Calculation: rewards / stake_amount
        //
        // Scaled by 1e9 to store 9 decimal places of precision.
        (rewards as u128)
            .checked_mul(REWARDS_PER_TOKEN_SCALING_FACTOR)
            .and_then(|product| product.checked_div(stake_amount as u128))
            .ok_or(ProgramError::ArithmeticOverflow)
    }
}

pub fn calculate_maximum_stake_for_lamports_amount(
    lamports_amount: u64,
) -> Result<u64, ProgramError> {
    if lamports_amount == 0 {
        Ok(0)
    } else {
        (lamports_amount as u128)
            .checked_mul(STAKE_FACTOR)
            .and_then(|product| product.checked_div(STAKE_SCALING_FACTOR))
            .and_then(|product| product.try_into().ok())
            .ok_or(ProgramError::ArithmeticOverflow)
    }
}

/// Struct to hold information about a delegation.
///
/// This holds the stake information for both `SolStakerStake` and `ValidatorStake`
/// accounts.
#[repr(C)]
#[derive(Clone, Copy, Default, Pod, ShankType, Zeroable)]
pub struct Delegation {
    /// Amount of staked tokens currently active.
    pub amount: u64,

    /// Timestamp for when deactivation began. Used to judge if a given stake
    /// is inactive.
    pub deactivation_timestamp: Option<NonZeroU64>,

    /// Amount of tokens in the cooling down phase, waiting to become inactive.
    pub deactivating_amount: u64,

    /// Amount that has passed the deactivation period, ready to be withdrawn.
    pub inactive_amount: u64,

    /// Authority permitted to deactivate and withdraw stake.
    pub authority: Pubkey,

    /// The address of the validator vote account to whom this stake is delegated.
    pub validator_vote: Pubkey,

    /// Stores the "last_seen_holder_rewards" just for this stake account, allowing
    /// stakers to withdraw rewards whenever, just like normal token users.
    pub last_seen_holder_rewards_per_token: PodU128,

    /// Stores the "last_seen_stake_rewards" just for this stake account, allowing
    /// stakers to withdraw rewards on their own schedule.
    pub last_seen_stake_rewards_per_token: PodU128,
}
