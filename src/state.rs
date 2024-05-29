//! Program state accounts

use {
    solana_program::clock::UnixTimestamp, solana_program::pubkey::Pubkey,
    spl_pod::optional_keys::OptionalNonZeroPubkey,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AccountType {
    Uninitialized,
    Config,
    Stake,
}

/// Configuration for a staking system
pub struct Config {
    /// After a deactivation, defines the number of seconds that must pass before
    /// the stake is inactive and able to be withdrawn
    cooldown_time_secs: u64,
    /// The maximum proportion that can be deactivated at once, given as basis
    /// points (1 / 10,000)
    max_deactivation_basis_points: u16,
    /// Total number of tokens delegated to the system. Since anyone can transfer
    /// tokens into the vault without passing through the program, this number
    /// is maintained independently.
    token_amount_delegated: u64,
    /// Authority that can modify any elements in the config
    config_authority: OptionalNonZeroPubkey,
    /// Authority that can slash any stake
    slash_authority: OptionalNonZeroPubkey,
    /// Token account storing all stake
    vault_token_account: Pubkey,
}

/// Data for an amount of tokens staked with a validator
pub struct Stake {
    /// Amount of staked tokens currently active
    amount: u64,
    /// Timestamp for when deactivation began. Used to judge if a given stake
    /// is inactive.
    /// NOTE: this is a special type where all zeros means `None` to avoid
    /// wasting space, just like `Option<NonZeroU64>`
    deactivation_timestamp: Option<UnixTimestamp>,
    /// Amount of tokens in the cooling down phase, waiting to become inactive
    deactivating_amount: u64,
    /// Amount that has passed the deactivation period, ready to be withdrawn
    inactive_amount: u64,
    /// Authority permitted to deactivate and withdraw stake
    authority: Pubkey,
    /// The address of the validator vote account (TODO this might not be needed)
    validator: Pubkey,
    /// Stores the "last_seen_holder_rewards" just for this stake account, allowing
    /// stakers to withdraw rewards whenever, just like normal token users
    last_seen_holder_rewards: u64,
    /// Stores the "last_seen_stake_rewards" just for this stake account, allowing
    /// stakers to withdraw rewards on their own schedule
    last_seen_stake_rewards: u64,
}
