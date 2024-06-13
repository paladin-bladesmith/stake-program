use bytemuck::{Pod, Zeroable};
use solana_program::pubkey::Pubkey;
use spl_pod::primitives::PodU64;

use super::AccountType;

/// Data for an amount of tokens staked with a validator
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Stake {
    /// Account type (must be `AccountType::Stake` when initialized).
    pub account_type: AccountType,

    /// Amount of staked tokens currently active
    pub amount: PodU64,

    /// Timestamp for when deactivation began. Used to judge if a given stake
    /// is inactive.
    /// NOTE: this is a special type where all zeros means `None` to avoid
    /// wasting space, just like `Option<NonZeroU64>`
    // TODO: Nullable trait
    pub deactivation_timestamp: PodU64,

    /// Amount of tokens in the cooling down phase, waiting to become inactive
    pub deactivating_amount: PodU64,

    /// Amount that has passed the deactivation period, ready to be withdrawn
    pub inactive_amount: PodU64,

    /// Authority permitted to deactivate and withdraw stake
    pub authority: Pubkey,

    /// The address of the validator vote account
    pub validator: Pubkey,

    /// Stores the "last_seen_holder_rewards" just for this stake account, allowing
    /// stakers to withdraw rewards whenever, just like normal token users
    pub last_seen_holder_rewards: PodU64,

    /// Stores the "last_seen_stake_rewards" just for this stake account, allowing
    /// stakers to withdraw rewards on their own schedule
    pub last_seen_stake_rewards: PodU64,
}

impl Stake {
    pub const LEN: usize = std::mem::size_of::<Stake>();
}
