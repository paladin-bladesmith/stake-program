use bytemuck::{Pod, Zeroable};
use shank::ShankAccount;
use solana_program::{program_pack::IsInitialized, pubkey::Pubkey};
use spl_discriminator::SplDiscriminate;
use spl_pod::primitives::PodU128;

use super::Delegation;

/// Data for an amount of tokens staked by a SOL staker.
///
/// A staker is able to stake tokens on the same validator where it is currently staking
/// SOL subject the maximum amount of tokens that can be staked being the minumum value
/// between:
///   * `lamports_amount * 1.3 * 0.5`; and
///   * `total_staked_sol_amount - total_staked_pal_amount` on the `ValidatorStake` account.
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
    pub lamports_amount: u64,

    /// The address of the SOL stake account.
    ///
    /// The `voter_pubkey` on the `StakeState` account must be equal to the `validator_vote`
    /// on the `delegation` struct.
    pub sol_stake: Pubkey,
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
                last_seen_holder_rewards_per_token: PodU128::default(),
                last_seen_stake_rewards_per_token: PodU128::default(),
            },
            lamports_amount: u64::default(),
            sol_stake: stake_state,
        }
    }
}

impl IsInitialized for SolStakerStake {
    fn is_initialized(&self) -> bool {
        self.is_initialized()
    }
}
