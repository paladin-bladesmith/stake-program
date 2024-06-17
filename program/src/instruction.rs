use shank::{ShankContext, ShankInstruction, ShankType};
use solana_program::clock::UnixTimestamp;

/// Enum defining all instructions in the Stake program.
#[rustfmt::skip]
#[derive(Clone, Debug, ShankContext, ShankInstruction)]
pub enum Instruction {
    /// Creates Stake config account which controls staking parameters.
    #[account(
        0,
        writable,
        name = "config",
        desc = "Stake config account"
    )]
    #[account(
        1,
        name = "config_authority",
        desc = "Config authority"
    )]
    #[account(
        2,
        name = "slash_authority",
        desc = "Slash authority"
    )]
    #[account(
        3,
        name = "mint",
        desc = "Stake token mint"
    )]
    #[account(
        4,
        name = "vault_token",
        desc = "Stake token vault"
    )]
    InitializeConfig {
        cooldown_time_seconds: UnixTimestamp,
        max_deactivation_basis_points: u16,
    },

    /// Initializes stake account data for a validator.
    /// 
    /// NOTE: Anybody can create the stake account for a validator. For new
    /// accounts, the authority is initialized to the validator vote account's
    /// withdraw authority.
    #[account(
        0,
        name = "config",
        desc = "Stake config account"
    )]
    #[account(
        1,
        writable,
        name = "stake",
        desc = "Validator stake account (pda of `['stake', validator, config]`)"
    )]
    #[account(
        2,
        name = "validator_vote",
        desc = "Validator vote account"
    )]
    #[account(
        3,
        name = "system_program",
        desc = "System program account"
    )]
    InitializeStake,

    /// Stakes tokens with the given config.
    /// 
    /// Limited to the current amount of SOL staked to the validator.
    ///
    /// NOTE: Anybody can stake tokens to a validator, but this does not work
    /// like native staking, because the validator can take control of staked
    /// tokens by deactivating and withdrawing.
    /// 
    /// Instruction data: amount of tokens to stake, as a little-endian `u64`.
    #[account(
        0,
        writable,
        name = "config",
        desc = "Stake config account"
    )]
    #[account(
        1,
        writable,
        name = "stake",
        desc = "Validator stake account"
    )]
    #[account(
        2,
        writable,
        name = "source_token",
        desc = "Token account"
    )]
    #[account(
        3,
        signer,
        name = "token_authority",
        desc = "Owner or delegate of the token account"
    )]
    #[account(
        4,
        name = "validator_vote",
        desc = "Validator vote account"
    )]
    #[account(
        5,
        name = "mint",
        desc = "Stake Token Mint"
    )]
    #[account(
        6,
        writable,
        name = "vault_token",
        desc = "Stake token Vault"
    )]
    #[account(
        7,
        name = "spl_token_program",
        desc = "SPL Token 2022 program"
    )]
    StakeTokens(u64),

    /// Deactivate staked tokens for the validator.
    /// 
    /// Only one deactivation may be in-flight at once, so if this is called
    /// with an active deactivation, it will succeed, but reset the amount and
    /// timestamp.
    /// 
    /// Instruction data: amount of tokens to deactivate, as a little-endian `u64``.
    #[account(
        0,
        writable,
        name = "stake",
        desc = "Validator stake account"
    )]
    #[account(
        1,
        signer,
        name = "stake_authority",
        desc = "Authority on validator stake account"
    )]
    DeactivateStake(u64),

    /// Move tokens from deactivating to inactive.
    /// 
    /// Reduces the total voting power for the stake account and the total staked
    /// amount on the system.
    /// 
    /// NOTE: This instruction is permissionless, so anybody can finish
    /// deactivating someone's tokens, preparing them to be withdrawn.
    #[account(
        0,
        writable,
        name = "config",
        desc = "Stake config account"
    )]
    #[account(
        1,
        writable,
        name = "stake",
        desc = "Validator stake account"
    )]
    InactivateStake,

    /// Withdraw inactive staked tokens from the vault.
    /// 
    /// After a deactivation has gone through the cooldown period and been
    /// "inactivated", the authority may move the tokens out of the vault.
    /// 
    /// Instruction data: amount of tokens to move.
    #[account(
        0,
        writable,
        name = "config",
        desc = "Stake config account"
    )]
    #[account(
        1,
        writable,
        name = "stake",
        desc = "Stake account"
    )]
    #[account(
        2,
        signer,
        name = "stake_authority",
        desc = "Stake authority"
    )]
    #[account(
        3,
        name = "vault_authority",
        desc = "Vault authority"
    )]
    #[account(
        4,
        writable,
        name = "vault_token",
        desc = "Vault token account"
    )]
    #[account(
        5,
        writable,
        name = "destination_token",
        desc = "Destination token account"
    )]
    #[account(
        6,
        name = "spl_token_program",
        desc = "SPL Token program"
    )]
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
    #[account(
        0,
        name = "config",
        desc = "Stake config account"
    )]
    #[account(
        1,
        writable,
        name = "stake",
        desc = "Stake account"
    )]
    #[account(
        2,
        writable,
        name = "vault_token",
        desc = "Vault token account"
    )]
    #[account(
        3,
        name = "vault_authority",
        desc = "Vault authority"
    )]
    #[account(
        4,
        name = "holder_rewards",
        desc = "Holder rewards account for vault token account"
    )]
    #[account(
        5,
        writable,
        name = "destination", 
        desc = "Destination account for withdrawn lamports"
    )]
    #[account(
        6,
        signer,
        name = "stake_authority",
        desc = "Stake authority"
    )]
    #[account(
        7,
        name = "mint",
        desc="Stake token mint"
    )]
    #[account(
        8,
        name = "spl_token_program",
        desc = "SPL Token program"
    )]
    HarvestHolderRewards,

    /// Harvests stake SOL rewards earned by the given stake account.
    /// 
    /// NOTE: This is very similar to the logic in the rewards program. Since the
    /// staking rewards are held in a separate account, they must be distributed
    /// based on the proportion of total stake.
    #[account(
        0,
        writable,
        name = "config",
        desc = "Stake config account"
    )]
    #[account(
        1,
        writable,
        name = "stake",
        desc = "Stake account"
    )]
    #[account(
        2,
        writable,
        name = "destination",
        desc = "Destination account for withdrawn lamports"
    )]
    #[account(
        3,
        signer,
        name = "stake_authority",
        desc = "Stake authority"
    )]
    HarvestStakeRewards,

    /// Slashes a stake account for the given amount.
    /// 
    /// Burns the given amount of tokens from the vault account, and reduces the
    /// amount in the stake account.
    /// 
    /// Instruction data: amount of tokens to slash.
    #[account(
        0,
        writable,
        name = "config",
        desc = "Stake config account"
    )]
    #[account(
        1,
        writable,
        name = "stake",
        desc = "Stake account"
    )]
    #[account(
        2,
        signer,
        name = "slash_authority",
        desc = "Stake account"
    )]
    #[account(
        3,
        writable,
        name = "vault_token",
        desc = "Vault token account"
    )]
    #[account(
        4,
        name = "vault_authority",
        desc = "Vault authority"
    )]
    #[account(
        5,
        name = "spl_token_program",
        desc = "SPL Token program"
    )]
    Slash(u64),

    /// Sets new authority on a config or stake account.
    #[account(
        0,
        writable,
        name = "account",
        desc = "Config or Stake config account"
    )]
    #[account(
        1,
        signer,
        name = "authority",
        desc = "Current authority on the account"
    )]
    #[account(
        2,
        name = "new_authority",
        desc = "Authority to set"
    )]
    SetAuthority(AuthorityType),

    /// Updates configuration parameters.
    #[account(
        0,
        writable,
        name = "config",
        desc = "Stake config account"
    )]
    #[account(
        1,
        signer,
        name = "config_authority",
        desc = "Stake config authority"
    )]
    UpdateConfig(ConfigField),

    /// Moves SOL rewards to the config and updates the stake rewards total.
    #[account(
        0,
        writable,
        name = "config",
        desc = "Stake config account"
    )]
    #[account(
        1,
        writable,
        signer,
        name = "payer",
        desc = "Reward payer"
    )]
    #[account(
        2,
        name = "system_program",
        desc = "System program account"
    )]
    DistributeRewards(u64),
}

/// Enum defining all authorities in the program
#[derive(Clone, Debug, Eq, PartialEq, ShankType)]
pub enum AuthorityType {
    Config,
    Slash,
    Stake,
}

/// Enum to allow updating the config account in the same instruction
#[derive(Clone, Debug, Eq, PartialEq, ShankType)]
pub enum ConfigField {
    /// Amount of seconds between deactivation and inactivation
    CooldownTimeSeconds(u64),
    /// Total proportion that can be deactivated at once, in basis points
    MaxDeactivationBasisPoints(u16),
}
