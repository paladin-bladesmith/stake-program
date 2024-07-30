use bytemuck::{Pod, Zeroable};
use shank::ShankAccount;
use solana_program::pubkey::Pubkey;
use spl_discriminator::SplDiscriminate;

use super::{Delegation, U128_DEFAULT};

/// Data for an amount of tokens staked by with a validator
#[repr(C)]
#[derive(Clone, Copy, Default, Pod, ShankAccount, SplDiscriminate, Zeroable)]
#[discriminator_hash_input("stake::state::sol_staker_stake")]
pub struct SolStakerStake {
    /// Account discriminator.
    ///
    /// The discriminator is equal to `ArrayDiscriminator:: UNINITIALIZED` when
    /// the account is empty, and equal to `SolStakerStake::DISCRIMINATOR` when the account
    /// is initialized.
    ///
    /// Note that the value of the discriminator is different than the prefix seed
    /// `"stake::state::sol_staker_stake"` used to derive the PDA address.
    discriminator: [u8; 8],

    /// Delegation values for the stake account.
    pub delegation: Delegation,

    /// Amount of SOL (lamports) staked on the stake state account.
    ///
    /// The maximum amount of tokens that can be staked is the minumum value between:
    ///   * lamports_amount * 1.3 * 0.5; and
    ///   * total_staked_sol_amount - total_staked_pal_amount on the `ValidatorStake` account.
    pub lamports_amount: u64,

    /// The address of the stake state account.
    ///
    /// The `voter_pubkey` on the `StakeState` account must be equal to the `validator_vote`
    /// on the base `Stake` struct.
    pub stake_state: Pubkey,
}

impl SolStakerStake {
    pub const LEN: usize = std::mem::size_of::<SolStakerStake>();

    /// Checks whether the discriminator has been set and it is equal to
    /// `SolStakerStake::SPL_DISCRIMINATOR_SLICE` or not.
    #[inline(always)]
    pub fn is_initialized(&self) -> bool {
        self.discriminator.as_slice() == SolStakerStake::SPL_DISCRIMINATOR_SLICE
    }

    /// Creates a new `SolStakerStake`.
    pub fn new(authority: Pubkey, stake_state: Pubkey, validator_vote: Pubkey) -> Self {
        Self {
            discriminator: SolStakerStake::SPL_DISCRIMINATOR.into(),
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
            lamports_amount: u64::default(),
            stake_state,
        }
    }
}
