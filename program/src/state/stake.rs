use std::num::NonZeroU64;

use bytemuck::{Pod, Zeroable};
use shank::ShankAccount;
use solana_program::pubkey::Pubkey;
use spl_discriminator::SplDiscriminate;

/// Data for an amount of tokens staked with a validator
#[repr(C)]
#[derive(Clone, Copy, Default, Pod, ShankAccount, SplDiscriminate, Zeroable)]
#[discriminator_hash_input("stake::state::stake")]
pub struct Stake {
    /// Account disciminator.
    ///
    /// The discriminator is equal to `ArrayDiscriminator:: UNINITIALIZED` when
    /// the account is empty, and equal to `Stake::DISCRIMINATOR` when the account
    /// is initialized.
    ///
    /// Note that the value of the discriminator is different than the prefix seed
    /// `"stake::state::stake"` used to derive the PDA address.
    discriminator: [u8; 8],

    /// Amount of staked tokens currently active
    pub amount: u64,

    /// Timestamp for when deactivation began. Used to judge if a given stake
    /// is inactive.
    pub deactivation_timestamp: Option<NonZeroU64>,

    /// Amount of tokens in the cooling down phase, waiting to become inactive
    pub deactivating_amount: u64,

    /// Amount that has passed the deactivation period, ready to be withdrawn
    pub inactive_amount: u64,

    /// Authority permitted to deactivate and withdraw stake
    pub authority: Pubkey,

    /// The address of the validator vote account
    pub validator: Pubkey,

    /// Stores the "last_seen_holder_rewards" just for this stake account, allowing
    /// stakers to withdraw rewards whenever, just like normal token users
    pub last_seen_holder_rewards: u64,

    /// Stores the "last_seen_stake_rewards" just for this stake account, allowing
    /// stakers to withdraw rewards on their own schedule
    pub last_seen_stake_rewards: u64,
}

impl Stake {
    pub const LEN: usize = std::mem::size_of::<Stake>();

    #[inline(always)]
    pub fn is_initialized(&self) -> bool {
        self.discriminator.as_slice() == Stake::SPL_DISCRIMINATOR_SLICE
    }

    pub fn new(authority: Pubkey, validator: Pubkey) -> Self {
        Self {
            discriminator: Stake::SPL_DISCRIMINATOR.into(),
            amount: u64::default(),
            deactivation_timestamp: Option::default(),
            deactivating_amount: u64::default(),
            inactive_amount: u64::default(),
            authority,
            validator,
            last_seen_holder_rewards: u64::default(),
            last_seen_stake_rewards: u64::default(),
        }
    }
}
