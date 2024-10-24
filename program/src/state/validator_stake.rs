use bytemuck::{Pod, Zeroable};
use shank::ShankAccount;
use solana_program::program_pack::IsInitialized;
use spl_discriminator::SplDiscriminate;

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
    pub _discriminator: [u8; 8],

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
        self._discriminator.as_slice() == ValidatorStake::SPL_DISCRIMINATOR_SLICE
    }
}

impl IsInitialized for ValidatorStake {
    fn is_initialized(&self) -> bool {
        self.is_initialized()
    }
}
