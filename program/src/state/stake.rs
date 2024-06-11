use bytemuck::{Pod, Zeroable};
use solana_program::pubkey::Pubkey;

/// Data for an amount of tokens staked with a validator
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Stake {
    // TODO: Account discriminator
    //
    /// Amount of staked tokens currently active
    amount: u64,

    /// Timestamp for when deactivation began. Used to judge if a given stake
    /// is inactive.
    /// NOTE: this is a special type where all zeros means `None` to avoid
    /// wasting space, just like `Option<NonZeroU64>`
    // TODO: Nullable trait
    deactivation_timestamp: u64,

    /// Amount of tokens in the cooling down phase, waiting to become inactive
    deactivating_amount: u64,

    /// Amount that has passed the deactivation period, ready to be withdrawn
    inactive_amount: u64,

    /// Authority permitted to deactivate and withdraw stake
    authority: Pubkey,

    /// The address of the validator vote account
    validator: Pubkey,

    /// Stores the "last_seen_holder_rewards" just for this stake account, allowing
    /// stakers to withdraw rewards whenever, just like normal token users
    last_seen_holder_rewards: u64,

    /// Stores the "last_seen_stake_rewards" just for this stake account, allowing
    /// stakers to withdraw rewards on their own schedule
    last_seen_stake_rewards: u64,
}
