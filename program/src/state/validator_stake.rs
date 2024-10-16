use bytemuck::{Pod, Zeroable};
use shank::ShankAccount;
use solana_program::{program_pack::IsInitialized, pubkey::Pubkey};
use spl_discriminator::SplDiscriminate;
use spl_pod::primitives::PodU128;

use super::Delegation;

/// Data for an amount of tokens staked by a validator.
///
/// The amount of tokens that a validator can stake is proportional to the amount of
/// SOL staked to the validator, subject to a limit equal to the minimum value between:
///   * `1.3 * 0.5 * total_staked_sol_amount`; and
///   * `total_staked_sol_amount - total_staked_pal_amount`
#[repr(C)]
#[derive(Clone, Copy, Default, Pod, ShankAccount, SplDiscriminate, Zeroable)]
#[discriminator_hash_input("stake::state::validator_stake")]
pub struct ValidatorStake {
    /// Account discriminator.
    ///
    /// The discriminator is equal to `ArrayDiscriminator::UNINITIALIZED` when
    /// the account is empty, and equal to `Stake::DISCRIMINATOR` when the account
    /// is initialized.
    ///
    /// Note that the value of the discriminator is different than the prefix seed
    /// `"stake::state::stake"` used to derive the PDA address.
    discriminator: [u8; 8],

    /// Delegation values for the stake account.
    pub delegation: Delegation,

    /// Total amount of SOL (lamports) staked on the validator.
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
                active_amount: u64::default(),
                effective_amount: u64::default(),
                deactivation_timestamp: Option::default(),
                deactivating_amount: u64::default(),
                inactive_amount: u64::default(),
                authority,
                validator_vote,
                last_seen_holder_rewards_per_token: PodU128::default(),
                last_seen_stake_rewards_per_token: PodU128::default(),
            },
            total_staked_lamports_amount: u64::default(),
        }
    }
}

impl IsInitialized for ValidatorStake {
    fn is_initialized(&self) -> bool {
        self.is_initialized()
    }
}
