use bytemuck::{Pod, Zeroable};
use shank::ShankAccount;
use solana_program::{clock::UnixTimestamp, pubkey::Pubkey};
use spl_discriminator::SplDiscriminate;
use spl_pod::optional_keys::OptionalNonZeroPubkey;

/// Configuration for a staking system.
#[repr(C)]
#[derive(Clone, Copy, Default, Pod, ShankAccount, SplDiscriminate, Zeroable)]
#[discriminator_hash_input("stake::state::config")]
pub struct Config {
    /// Account disciminator.
    ///
    /// The discriminator is equal to `ArrayDiscriminator:: UNINITIALIZED` when
    /// the account is empty, and equal to `Config::DISCRIMINATOR` when the account
    /// is initialized.
    pub(crate) discriminator: [u8; 8],

    /// Authority that can modify any elements in the config.
    pub authority: OptionalNonZeroPubkey,

    /// Authority that can slash any stake.
    pub slash_authority: OptionalNonZeroPubkey,

    /// Token account storing all stake.
    pub vault: Pubkey,

    /// After a deactivation, defines the number of seconds that must pass before
    /// the stake is inactive and able to be withdrawn.
    pub cooldown_time_seconds: UnixTimestamp,

    /// Total number of tokens delegated to the system. Since anyone can transfer
    /// tokens into the vault without passing through the program, this number
    /// is maintained independently.
    pub token_amount_delegated: u64,

    /// Running total of all stake rewards distributed.
    pub total_stake_rewards: u64,

    /// The maximum proportion that can be deactivated at once, given as basis
    /// points (1 / 10,000).
    pub max_deactivation_basis_points: u16,

    /// Padding for alignment.
    pub(crate) _padding: [u8; 6],
}

impl Config {
    pub const LEN: usize = std::mem::size_of::<Config>();

    #[inline(always)]
    pub fn is_initialized(&self) -> bool {
        self.discriminator.as_slice() == Config::SPL_DISCRIMINATOR_SLICE
    }

    #[inline(always)]
    pub fn get_signer_bump(&self) -> u8 {
        self._padding[0]
    }

    #[inline(always)]
    pub fn set_signer_bump(&mut self, bump: u8) {
        self._padding[0] = bump;
    }
}
