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
    discriminator: [u8; 8],

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
    _padding: [u8; 6],
}

impl Config {
    pub const LEN: usize = std::mem::size_of::<Config>();
}
