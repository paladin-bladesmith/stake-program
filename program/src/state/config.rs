use bytemuck::{Pod, Zeroable};
use shank::ShankAccount;
use solana_program::pubkey::Pubkey;
use spl_discriminator::{ArrayDiscriminator, SplDiscriminate};
use spl_pod::{optional_keys::OptionalNonZeroPubkey, primitives::PodU128};

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
    discriminator: [u8; 8],

    /// Authority that can modify any elements in the config.
    pub authority: OptionalNonZeroPubkey,

    /// Authority that can slash any stake.
    pub slash_authority: OptionalNonZeroPubkey,

    /// Token account storing all stake.
    pub vault: Pubkey,

    /// After a deactivation, defines the number of seconds that must pass before
    /// the stake is inactive and able to be withdrawn.
    pub cooldown_time_seconds: u64,

    /// Total number of tokens delegated to the system. Since anyone can transfer
    /// tokens into the vault without passing through the program, this number
    /// is maintained independently.
    pub token_amount_delegated: u64,

    /// The current stake rewards per token exchange rate.
    ///
    /// Stored as a `u128`, which includes a scaling factor of `1e9` to
    /// represent the exchange rate with 9 decimal places of precision.
    pub accumulated_stake_rewards_per_token: PodU128,

    /// The maximum proportion that can be deactivated at once, given as basis
    /// points (1 / 10,000).
    pub max_deactivation_basis_points: u16,

    /// Bump seed for the `Vault` signer authority.
    pub vault_authority_bump: u8,

    /// Padding for alignment.
    _padding: [u8; 5],
}

impl Config {
    pub const LEN: usize = std::mem::size_of::<Config>();

    pub fn new(
        authority: OptionalNonZeroPubkey,
        slash_authority: OptionalNonZeroPubkey,
        vault: Pubkey,
        cooldown_time_seconds: u64,
        max_deactivation_basis_points: u16,
        vault_authority_bump: u8,
    ) -> Self {
        Self {
            discriminator: Config::SPL_DISCRIMINATOR.into(),
            authority,
            slash_authority,
            vault,
            cooldown_time_seconds,
            token_amount_delegated: 0,
            accumulated_stake_rewards_per_token: PodU128::default(),
            max_deactivation_basis_points,
            vault_authority_bump,
            _padding: [0; 5],
        }
    }

    #[inline(always)]
    pub fn is_initialized(&self) -> bool {
        self.discriminator.as_slice() == Config::SPL_DISCRIMINATOR_SLICE
    }

    #[inline(always)]
    pub fn is_uninitialized(&self) -> bool {
        self.discriminator.as_slice() == ArrayDiscriminator::UNINITIALIZED.as_slice()
    }
}
