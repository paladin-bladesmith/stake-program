//! Program instructions

use solana_program::clock::UnixTimestamp;

/// Instructions supported by the staking program
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StakeInstruction {
    /// Creates Stake config account which controls staking parameters
    ///
    /// 0. `[w]` Stake config account
    /// 1. `[]` Slash authority, may be a PDA for a governance program
    /// 2. `[]` Config authority, may be a PDA for a governance program
    /// 3. `[]` Stake Token Mint
    /// 4. `[]` Stake Token Vault, to hold all staked tokens.
    ///   Must be empty and owned by PDA with seeds `['token-owner', stake_config]`
    InitializeConfig {
        cooldown_time: UnixTimestamp,
        max_deactivation_basis_points: u16,
    },

    /// Initializes stake account data for a validator.
    ///
    /// NOTE: Anybody can create the stake account for a validator. For new
    /// accounts, the authority is initialized to the validator vote account's
    /// withdraw authority.
    ///
    /// 0. `[]` Stake config account
    /// 1. `[w]` Validator stake account
    ///     * PDA seeds: ['stake', validator, config_account]
    /// 2. `[]` Validator vote account
    /// 3. `[]` System program
    InitializeStake,

    /// Stakes tokens with the given config.
    ///
    /// Limited to the current amount of SOL staked to the validator.
    ///
    /// NOTE: Anybody can stake tokens to a validator, but this does not work
    /// like native staking, because the validator can take control of staked
    /// tokens by deactivating and withdrawing.
    ///
    /// 0. `[w]` Stake config account
    /// 1. `[w]` Validator stake account
    ///     * PDA seeds: ['stake', validator, config_account]
    /// 2. `[w]` Token Account
    /// 3. `[s]` Owner or delegate of the token account
    /// 4. `[]` Validator vote account
    /// 3. `[]` Stake Token Mint
    /// 4. `[]` Stake Token Vault, to hold all staked tokens.
    ///   Must be the token account on the stake config account
    /// 5. `[]` Token program
    /// 6.. Extra accounts required for the transfer hook
    ///
    /// Instruction data: amount of tokens to stake, as a little-endian u64
    StakeTokens(u64),

    /// Deactivate staked tokens for the validator.
    ///
    /// Only one deactivation may be in-flight at once, so if this is called
    /// with an active deactivation, it will succeed, but reset the amount and
    /// timestamp.
    ///
    /// 0. `[w]` Validator stake account
    /// 1. `[s]` Authority on validator stake account
    ///
    /// Instruction data: amount of tokens to deactivate, as a little-endian u64
    DeactivateStake(u64),

    /// Move tokens from deactivating to inactive.
    ///
    /// Reduces the total voting power for the stake account and the total staked
    /// amount on the system.
    ///
    /// NOTE: This instruction is permissionless, so anybody can finish
    /// deactivating someone's tokens, preparing them to be withdrawn.
    ///
    /// 0. `[w]` Stake config account
    /// 1. `[w]` Validator stake account
    InactivateStake,

    /// Withdraw inactive staked tokens from the vault
    ///
    /// After a deactivation has gone through the cooldown period and been
    /// "inactivated", the authority may move the tokens out of the vault.
    ///
    /// 0. `[w]` Config account
    /// 1. `[w]` Stake account
    /// 2. `[w]` Vault token account
    /// 3. `[w]` Destination token account
    /// 4. `[s]` Stake authority
    /// 5. `[]` Vault authority, PDA with seeds `['token-owner', stake_config]`
    /// 6. `[]` SPL Token program
    /// 7.. Extra required accounts for transfer hook
    ///
    /// Instruction data: amount of tokens to move
    WithdrawInactiveStake(u64),

    /// Harvests holder SOL rewards earned by the given stake account.
    ///
    /// NOTE: This mostly replicates the logic in the rewards program. Since the
    /// staked tokens are all held by this program, stakers need a way to access
    /// their portion of holder rewards.
    ///
    /// This instruction requires that `unclaimed_rewards` be equal to `0` in
    /// the token vault account. For ease of use, be sure to call the
    /// `HarvestRewards` on the vault account before this.
    ///
    /// 0. `[]` Config account
    /// 1. `[w]` Stake account
    /// 2. `[w]` Vault token account
    /// 3. `[]` Holder rewards account for vault token account
    /// 4. `[w]` Destination account for withdrawn lamports
    /// 5. `[s]` Stake authority
    /// 6. `[]` Vault authority, PDA with seeds `['token-owner', stake_config]`
    /// 7. `[]` Stake token mint, to get total supply
    /// 8. `[]` SPL Token program
    HarvestHolderRewards,

    /// Harvests stake SOL rewards earned by the given stake account.
    ///
    /// NOTE: This is very similar to the logic in the rewards program. Since the
    /// staking rewards are held in a separate account, they must be distributed
    /// based on the proportion of total stake.
    ///
    /// 0. `[]` Config account
    /// 1. `[w]` Stake account
    /// 2. `[w]` SOL staking rewards account
    ///    (TODO for discussion: we need a way to also track total staking rewards
    ///    so stakers can know their allotted proportion of staking rewards,
    ///    which are separate from the holder rewards. This means that the distribution
    ///    logic in the Rewards program *also* needs to update some running total
    ///    of staking rewards. I couldn't come up with a way to combine these two,
    ///    since the proportion allocated to the different groups is *not* meant
    ///    to be fixed forever.)
    /// 3. `[s]` Stake authority
    /// 4. `[]` Staking rewards authority
    ///    (TODO per the above point, what should this be? Some PDA for this
    ///    program?)
    /// 6. `[]` Rewards program
    HarvestStakeRewards,

    /// Slashes a stake account for the given amount
    ///
    /// Burns the given amount of tokens from the vault account, and reduces the
    /// amount in the stake account.
    ///
    /// 0. `[w]` Config account
    /// 1. `[w]` Stake account
    /// 2. `[s]` Slash authority
    /// 3. `[w]` Vault token account
    /// 4. `[]` Vault authority, PDA with seeds `['token-owner', stake_config]`
    /// 5. `[]` SPL Token program
    ///
    /// Instruction data: amount of tokens to slash
    Slash(u64),

    /// Sets new authority on a config or stake account
    ///
    /// 0. `[w]` Config or stake account
    /// 1. `[s]` Current authority
    /// 2. `[]` New authority
    SetAuthority(Authority),

    /// Updates configuration parameters
    ///
    /// 0. `[w]` Config account
    /// 1. `[s]` Config authority
    UpdateConfig(ConfigField),
}

/// Enum defining all authorities in the program
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Authority {
    Config,
    Slash,
    Stake,
}

/// Enum to allow updating the config account in the same instruction
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConfigField {
    /// Amount of seconds between deactivation and inactivation
    CooldownTimeSecs(u64),
    /// Total proportion that can be deactivated at once, in basis points
    MaxDeactivationBasisPoints(u16),
}
