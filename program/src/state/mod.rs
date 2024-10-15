pub mod config;
pub mod sol_staker_stake;
pub mod validator_stake;

pub use config::*;
use paladin_sol_stake_view_program_client::GetStakeActivatingAndDeactivatingReturnData;
pub use sol_staker_stake::*;
use spl_pod::primitives::PodU128;
pub use validator_stake::*;

use bytemuck::{Pod, Zeroable};
use shank::ShankType;
use solana_program::{
    program_error::ProgramError,
    pubkey::{Pubkey, PubkeyError},
};
use std::{mem::size_of, num::NonZeroU64};

/// Scaling factor for rewards per token (1e18).
const REWARDS_PER_TOKEN_SCALING_FACTOR: u128 = 1_000_000_000_000_000_000;

/// Defined the maximum value for basis points (100%).
pub const MAX_BASIS_POINTS: u128 = 10_000;

/// Stake factor for the maximum amount of staked tokens as a percentage of the total
/// SOL staked (STAKE_FACTOR / STAKE_SCALING_FACTOR).
pub const STAKE_FACTOR: u128 = 13;

/// Scaling factor for stake amount.
pub const STAKE_SCALING_FACTOR: u128 = 10;

/// Represents a return data with no delegated values.
pub const EMPTY_RETURN_DATA: [u8; size_of::<GetStakeActivatingAndDeactivatingReturnData>()] =
    [0; size_of::<GetStakeActivatingAndDeactivatingReturnData>()];

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
    let marginal_rate =
        current_accumulated_rewards_per_token.wrapping_sub(last_accumulated_rewards_per_token);

    if marginal_rate == 0 {
        Ok(0)
    } else {
        // Scaled by 1e18 to store 18 decimal places of precision.
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
        // Scaled by 1e18 to store 18 decimal places of precision.
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

    // TODO: Amount does not include inactive amount but amount is used for holder rewards (which inactive tokens should still earn)...
    /// Amount of staked tokens (but capped at 1.3 PAL per SOL).
    pub effective_amount: u64,

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

#[cfg(test)]
mod tests {
    use super::*;

    const BENCH_TOKEN_SUPPLY: u64 = 1_000_000_000 * 1_000_000_000; // 1 billion with 9 decimals

    #[test]
    fn minimum_stake_rewards_per_token() {
        // 1 lamport (arithmetic minimum)
        let minimum_reward = 1;
        let result = calculate_stake_rewards_per_token(minimum_reward, BENCH_TOKEN_SUPPLY).unwrap();
        assert_ne!(result, 0);

        // Anything below the minimum should return zero.
        let result =
            calculate_stake_rewards_per_token(minimum_reward - 1, BENCH_TOKEN_SUPPLY).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn maximum_stake_rewards_per_token() {
        // u64::MAX (not really practical, but shows that we're ok)
        let maximum_reward = u64::MAX;
        let _ = calculate_stake_rewards_per_token(maximum_reward, BENCH_TOKEN_SUPPLY).unwrap();
    }

    #[test]
    fn minimum_eligible_rewards() {
        // 1 / 1e18 lamports per token
        let minimum_marginal_rewards_per_token = 1;
        let result = calculate_eligible_rewards(
            minimum_marginal_rewards_per_token,
            0,
            BENCH_TOKEN_SUPPLY, // 100% of the supply.
        )
        .unwrap();
        assert_ne!(result, 0);

        // Anything below the minimum should return zero.
        let result = calculate_eligible_rewards(
            minimum_marginal_rewards_per_token - 1,
            0,
            BENCH_TOKEN_SUPPLY, // 100% of the supply.
        )
        .unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn minimum_eligible_rewards_with_one_token() {
        // 1 / 1e9 lamports per token
        let minimum_marginal_rewards_per_token = 1_000_000_000;
        let result = calculate_eligible_rewards(
            minimum_marginal_rewards_per_token,
            0,
            BENCH_TOKEN_SUPPLY / 1_000_000_000, // 1 with 9 decimals.
        )
        .unwrap();
        assert_ne!(result, 0);

        // Anything below the minimum should return zero.
        let result = calculate_eligible_rewards(
            minimum_marginal_rewards_per_token - 1,
            0,
            BENCH_TOKEN_SUPPLY / 1_000_000_000, // 1 with 9 decimals.
        )
        .unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn minimum_eligible_rewards_with_smallest_fractional_token() {
        // 1 lamport per token
        let minimum_marginal_rewards_per_token = 1_000_000_000_000_000_000;
        let result = calculate_eligible_rewards(
            minimum_marginal_rewards_per_token,
            0,
            BENCH_TOKEN_SUPPLY / 1_000_000_000_000_000_000, // .000_000_001 with 9 decimals.
        )
        .unwrap();
        assert_ne!(result, 0);

        // Anything below the minimum should return zero.
        let result = calculate_eligible_rewards(
            minimum_marginal_rewards_per_token - 1,
            0,
            BENCH_TOKEN_SUPPLY / 1_000_000_000_000_000_000, // .000_000_001 with 9 decimals.
        )
        .unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn maximum_eligible_rewards() {
        // 1 lamport per token (not really practical, but shows that we're ok)
        let maximum_marginal_rewards_per_token = REWARDS_PER_TOKEN_SCALING_FACTOR;
        let _ = calculate_eligible_rewards(
            maximum_marginal_rewards_per_token,
            0,
            BENCH_TOKEN_SUPPLY, // 100% of the supply.
        )
        .unwrap();
    }

    #[test]
    fn wrapping_eligible_rewards() {
        // Set up current to be less than rate, simulating a scenario where the
        // current reward has wrapped around `u128::MAX`.
        let current_accumulated_rewards_per_token = 1_000_000_000_000_000_000;
        let last_accumulated_rewards_per_token = u128::MAX - 1_000_000_000_000_000_000;
        let result = calculate_eligible_rewards(
            current_accumulated_rewards_per_token,
            last_accumulated_rewards_per_token,
            BENCH_TOKEN_SUPPLY,
        )
        .unwrap();
        assert_eq!(result, 2_000_000_000_000_000_001);

        // Try it again at the very edge. Result should be one.
        let current_accumulated_rewards_per_token = 0;
        let last_accumulated_rewards_per_token = u128::MAX;
        let result = calculate_eligible_rewards(
            current_accumulated_rewards_per_token,
            last_accumulated_rewards_per_token,
            BENCH_TOKEN_SUPPLY,
        )
        .unwrap();
        assert_eq!(result, 1);
    }
}
