use bytemuck::{Pod, Zeroable};
use solana_program::pubkey::Pubkey;
use spl_pod::{
    optional_keys::OptionalNonZeroPubkey,
    primitives::{PodU16, PodU64},
};

use super::AccountType;

/// Configuration for a staking system.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Config {
    /// Account type (must be `AccountType::Config` when initialized).
    pub account_type: AccountType,

    /// Authority that can modify any elements in the config.
    pub authority: OptionalNonZeroPubkey,

    /// Authority that can slash any stake.
    pub slash_authority: OptionalNonZeroPubkey,

    /// Token account storing all stake.
    pub vault_token: Pubkey,

    /// Bump seed for the vault account.
    pub vault_bump: u8,

    /// After a deactivation, defines the number of seconds that must pass before
    /// the stake is inactive and able to be withdrawn.
    pub cooldown_time_seconds: PodU64,

    /// The maximum proportion that can be deactivated at once, given as basis
    /// points (1 / 10,000).
    pub max_deactivation_basis_points: PodU16,

    /// Total number of tokens delegated to the system. Since anyone can transfer
    /// tokens into the vault without passing through the program, this number
    /// is maintained independently.
    pub token_amount_delegated: PodU64,

    /// Running total of all stake rewards distributed.
    pub total_stake_rewards: PodU64,
}

impl Config {
    pub const LEN: usize = std::mem::size_of::<Config>();
}
