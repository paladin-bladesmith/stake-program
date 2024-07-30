use bytemuck::{Pod, Zeroable};
use shank::ShankAccount;
use solana_program::pubkey::Pubkey;
use spl_discriminator::SplDiscriminate;

use super::{Delegation, U128_DEFAULT};

/// Data for an amount of tokens staked with a validator.
///
/// This account represents the stake of a validator.
#[repr(C)]
#[derive(Clone, Copy, Default, Pod, ShankAccount, SplDiscriminate, Zeroable)]
#[discriminator_hash_input("stake::state::validator_stake")]
pub struct ValidatorStake {
    /// Account disciminator.
    ///
    /// The discriminator is equal to `ArrayDiscriminator:: UNINITIALIZED` when
    /// the account is empty, and equal to `Stake::DISCRIMINATOR` when the account
    /// is initialized.
    ///
    /// Note that the value of the discriminator is different than the prefix seed
    /// `"stake::state::stake"` used to derive the PDA address.
    discriminator: [u8; 8],

    /// Delegation values for the stake account.
    pub delegation: Delegation,

    /// Total amount of PAL tokens staked on validator stake account.
    ///
    /// The total includes the amount staked by the validator and the amount staked by
    /// all `SolStakerStake` accounts delegating to the validator.
    pub total_staked_pal_amount: u64,

    /// Total amount of SOL (lamports) staked on validator stake account.
    ///
    /// The total includes the amount staked by the validator and the amount staked by
    /// all `SolStakerStake` accounts delegating to the validator.
    pub total_staked_lamports_amount: u64,
}

impl ValidatorStake {
    pub const LEN: usize = std::mem::size_of::<ValidatorStake>();

    /// Checks whether the discriminator has been set and it is equal to
    /// `ValidatorStake::SPL_DISCRIMINATOR_SLICE` or not.
    #[inline(always)]
    pub fn is_initialized(&self) -> bool {
        self.discriminator.as_slice() == ValidatorStake::SPL_DISCRIMINATOR_SLICE
    }

    /// Creates a new `ValidatorStake`.
    pub fn new(authority: Pubkey, validator_vote: Pubkey) -> Self {
        Self {
            discriminator: ValidatorStake::SPL_DISCRIMINATOR.into(),
            delegation: Delegation {
                amount: u64::default(),
                deactivation_timestamp: Option::default(),
                deactivating_amount: u64::default(),
                inactive_amount: u64::default(),
                authority,
                validator_vote,
                last_seen_holder_rewards_per_token: U128_DEFAULT,
                last_seen_stake_rewards_per_token: U128_DEFAULT,
            },
            total_staked_pal_amount: u64::default(),
            total_staked_lamports_amount: u64::default(),
        }
    }
}
