use borsh::{BorshDeserialize, BorshSerialize};
use shank::{ShankContext, ShankInstruction};

/// Enum defining all instructions in the Stake program.
#[rustfmt::skip]
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, ShankContext, ShankInstruction)]
pub enum Instruction {
    /// Creates Stake config account which controls staking parameters.
    #[account(0, signer, writable, name="config", desc="Stake config account")]
    #[account(1, name="config_authority", desc="Config authority")]
    #[account(2, name="slash_authority", desc="Slash authority")]
    #[account(3, name="mint", desc="Stake token mint")]
    #[account(4, name="vault_token", desc="Stake token vault")]
    InitializeConfig {
        cooldown_time_seconds: u64,
        max_deactivation_basis_points: u16,
    },

    /// Initializes stake account data for a validator.
    #[account(0, name="config", desc="Stake config account")]
    #[account(1, writable, name="stake", desc="Validator stake account (pda of `['stake', config, validator]`)")]
    #[account(2, name="validator_vote", desc="Validator vote account")]
    #[account(3, name="system_program", desc="System program account")]
    InitializeStake,

    /// Stakes tokens with the given config.
    #[account(0, writable, name="config", desc="Stake config account")]
    #[account(1, writable, name="stake", desc="Validator stake account")]
    #[account(2, writable, name="source_token", desc="Token account")]
    #[account(3, signer, name="token_authority", desc="Owner or delegate of the token account")]
    #[account(4, name="validator_vote", desc="Validator vote account")] // <- needed?
    #[account(5, name="mint", desc="Stake Token Mint")]
    #[account(6, writable, name="vault_token", desc="Stake token Vault")]
    #[account(7, name="spl_token_program", desc="SPL Token 2022 program")]
    StakeTokens(u64),

    /// Deactivate staked tokens for the validator.
    #[account(0, writable, name="stake", desc="Validator stake account")]
    #[account(1, signer, name="stake_authority", desc="Authority on validator stake account")]
    DeactivateStake(u64),

    /// Move tokens from deactivating to inactive.
    #[account(0, writable, name="config", desc="Stake config account")]
    #[account(1, writable, name="stake", desc="Validator stake account")]
    InactivateStake,

    /// Withdraw inactive staked tokens from the vault.
    #[account(0, writable, name="config", desc="Stake config account")]
    #[account(1, writable, name="stake", desc="Stake account")]
    #[account(2, signer, name="stake_authority", desc="Stake authority")]
    #[account(3, name="vault_authority", desc="Vault authority")]
    #[account(4, writable, name="vault_token", desc="Vault token account")]
    #[account(5, writable, name="destination_token", desc="Destination token account")]
    #[account(6, name="spl_token_program", desc="SPL Token program")]
    WithdrawInactiveStake(u64),

    /// Harvests holder SOL rewards earned by the given stake account.
    #[account(0, name="config", desc="Stake config account")]
    #[account(1, writable, name="stake", desc="Stake account")]
    #[account(2, writable, name="vault_token", desc="Vault token account")]
    #[account(3, name="vault_authority", desc="Vault authority")]
    #[account(4, name="holder_rewards", desc="Holder rewards account for vault token account")]
    #[account(5, writable, name="destination", desc="Destination account for withdrawn lamports")]
    #[account(6, signer, name="stake_authority", desc="Stake authority")]
    #[account(7, name="mint", desc="Stake token mint")]
    #[account(8, name="spl_token_program", desc="SPL Token program")]
    HarvestHolderRewards,

    /// Harvests stake SOL rewards earned by the given stake account.
    #[account(0, writable, name="config", desc="Stake config account")]
    #[account(1, writable, name="stake", desc="Stake account")]
    #[account(2, writable, name="destination", desc="Destination account for withdrawn lamports")]
    #[account(3, signer, name="stake_authority", desc="Stake authority")]
    HarvestStakeRewards,

    /// Slashes a stake account for the given amount
    #[account(0, writable, name="config", desc="Stake config account")]
    #[account(1, writable, name="stake", desc="Stake account")]
    #[account(2, signer, name="slash_authority", desc="Stake account")]
    #[account(3, writable, name="vault_token", desc="Vault token account")]
    #[account(4, name="vault_authority", desc="Vault authority")]
    #[account(5, name="spl_token_program", desc="SPL Token program")]
    Slash(u64),

    /// Sets new authority on a config or stake account
    #[account(0, writable, name="account", desc="Config or Stake config account")]
    #[account(1, signer, name="authority", desc="Current authority on the account")]
    #[account(2, name="new_authority", desc="Authority to set")]
    SetAuthority(AuthorityType),

    /// Updates configuration parameters
    #[account(0, writable, name="config", desc="Stake config account")]
    #[account(1, signer, name="config_authority", desc="Stake config authority")]
    UpdateConfig(ConfigField),

    /// Moves SOL rewards to the config and updates the stake rewards total
    #[account(0, writable, name="config", desc="Stake config account")]
    #[account(1, writable, signer, name="payer", desc="Reward payer")]
    #[account(2, name="system_program", desc="System program account")]
    DistributeRewards(u64),
}

/// Enum defining all authorities in the program
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq, Eq)]
pub enum AuthorityType {
    Config,
    Slash,
    Stake,
}

/// Enum to allow updating the config account in the same instruction
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq, Eq)]
pub enum ConfigField {
    /// Amount of seconds between deactivation and inactivation
    CooldownTimeSeconds(u64),
    /// Total proportion that can be deactivated at once, in basis points
    MaxDeactivationBasisPoints(u16),
}
