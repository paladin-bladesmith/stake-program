use arrayref::array_ref;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use shank::{ShankContext, ShankInstruction, ShankType};
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

/// Enum defining all instructions in the Stake program.
#[repr(C)]
#[derive(Clone, Debug, Eq, PartialEq, ShankContext, ShankInstruction)]
#[rustfmt::skip]
pub enum StakeInstruction {
    /// Creates Stake config account which controls staking parameters.
    #[account(
        0,
        writable,
        name = "config",
        desc = "Stake config account"
    )]
    #[account(
        1,
        name = "mint",
        desc = "Stake token mint"
    )]
    #[account(
        2,
        name = "vault",
        desc = "Stake vault token account"
    )]
    InitializeConfig {
        slash_authority: Pubkey,
        config_authority: Pubkey,
        cooldown_time_seconds: u64,
        max_deactivation_basis_points: u16,
        sync_rewards_lamports: u64,
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
        name = "validator_stake",
        desc = "Validator stake account (pda of `['stake::state::validator_stake', validator, config]`)"
    )]
    #[account(
        2,
        name = "validator_vote",
        desc = "Validator vote account"
    )]
    #[account(
        3,
        name = "system_program",
        desc = "System program"
    )]
    InitializeValidatorStake,

    /// Stakes tokens with the given config.
    ///
    /// NOTE: This instruction is used by validator stake accounts. The total amount of staked
    /// tokens is limited to the 1.3 * current amount of SOL staked to the validator.
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
        name = "validator_stake",
        desc = "Validator stake account (pda of `['stake::state::validator_stake', validator, config]`)"
    )]
    #[account(
        2,
        writable,
        name = "validator_stake_authority",
        desc = "Validator stake account (pda of `['stake::state::validator_stake', validator, config]`)"
    )]
    #[account(
        3,
        writable,
        name = "source_token_account",
        desc = "Token account"
    )]
    #[account(
        4,
        signer,
        name = "token_account_authority",
        desc = "Owner or delegate of the token account"
    )]
    #[account(
        5,
        name = "mint",
        desc = "Stake Token Mint"
    )]
    #[account(
        6,
        writable,
        name = "vault",
        desc = "Stake token Vault"
    )]
    #[account(
        7,
        writable,
        name = "vault_holder_rewards",
        desc = "Holder rewards for the vault account (to facilitate harvest)"
    )]
    #[account(
        8,
        name = "token_program",
        desc = "Token program"
    )]
    ValidatorStakeTokens(u64),

    /// Deactivate staked tokens for a stake delegation.
    ///
    /// Only one deactivation may be in-flight at once, so if this is called
    /// with an active deactivation, it will succeed, but reset the amount and
    /// timestamp.
    ///
    /// Instruction data: amount of tokens to deactivate, as a little-endian `u64`.
    #[account(
        0,
        name = "config",
        desc = "Stake config account"
    )]
    #[account(
        1,
        writable,
        name = "stake",
        desc = "Validator or SOL staker stake account"
    )]
    #[account(
        2,
        signer,
        name = "stake_authority",
        desc = "Authority on validator stake account"
    )]
    DeactivateStake(u64),

    /// Move tokens from deactivating to inactive.
    ///
    /// Reduces the total voting power for the validator stake account and the total staked
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
        name = "validator_stake",
        desc = "Validator stake account (pda of `['stake::state::validator_stake', validator, config]`)"
    )]
    #[account(
        2,
        writable,
        name = "validator_stake_authority",
        desc = "Validator stake authority account"
    )]
    #[account(
        3,
        writable,
        name = "vault_holder_rewards",
        desc = "Vault holder rewards account"
    )]
    InactivateValidatorStake,

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
        desc = "Validator or SOL staker stake account"
    )]
    #[account(
        2,
        writable,
        name = "vault",
        desc = "Vault token account"
    )]
    #[account(
        3,
        name = "mint",
        desc = "Stake Token Mint"
    )]
    #[account(
        4,
        writable,
        name = "destination_token_account",
        desc = "Destination token account"
    )]
    #[account(
        5,
        signer,
        name = "stake_authority",
        desc = "Stake authority"
    )]
    #[account(
        6,
        name = "vault_authority",
        desc = "Vault authority (pda of `['token-owner', config]`)"
    )]

    #[account(
        7,
        name = "token_program",
        desc = "Token program"
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
        writable,
        name = "config",
        desc = "Stake config account"
    )]
    #[account(
        1,
        writable,
        name = "holder_rewards_pool",
        desc = "Holder rewards pool account"
    )]
    #[account(
        2,
        writable,
        name = "vault",
        desc = "Vault token account"
    )]
    #[account(
        3,
        writable,
        name = "vault_holder_rewards",
        desc = "Holder rewards account for vault token account"
    )]
    #[account(
        4,
        name = "vault_authority",
        desc = "Vault authority (pda of `['token-owner', config]`)"
    )]
    #[account(
        5,
        name = "mint",
        desc = "Stake token mint"
    )]
    #[account(
        6,
        name = "token_program",
        desc = "Token program"
    )]
    #[account(
        7,
        name = "paladin_rewards_program",
        desc = "Paladin rewards program"
    )]
    #[account(
        8,
        name = "system_program",
        desc = "System program"
    )]
    HarvestHolderRewards,

    /// Harvests stake SOL rewards earned by the given validator stake account.
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
        name = "vault_holder_rewards",
        desc = "Holder rewards account"
    )]
    #[account(
        2,
        writable,
        name = "validator_stake",
        desc = "Validator stake account"
    )]
    #[account(
        3,
        writable,
        name = "validator_stake_authority",
        desc = "Validator stake authority"
    )]
    HarvestValidatorRewards,

    /// Slashes a validator stake account for the given amount.
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
        name = "validator_stake",
        desc = "Validator stake account (pda of `['stake::state::validator_stake', validator, config]`)"
    )]
    #[account(
        2,
        writable,
        name = "validator_stake_authority",
        desc = "Validator stake authority account"
    )]
    #[account(
        3,
        signer,
        name = "slash_authority",
        desc = "Config slash authority"
    )]
    #[account(
        4,
        writable,
        name = "vault",
        desc = "Vault token account"
    )]
    #[account(
        5,
        name = "vault_holder_rewards",
        desc = "Vault token account"
    )]
    #[account(
        6,
        name = "vault_authority",
        desc = "Vault authority (pda of `['token-owner', config]`)"
    )]
    #[account(
        7,
        writable,
        name = "mint",
        desc = "Stake Token Mint"
    )]
    #[account(
        8,
        name = "token_program",
        desc = "Token program"
    )]
    SlashValidatorStake(u64),

    /// Sets new authority on a config or stake account.
    #[account(
        0,
        writable,
        name = "account",
        desc = "Config or Stake account"
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

    /// Initializes stake account data for a SOL staker.
    ///
    /// NOTE: Anybody can create the stake account for a SOL staker. For new
    /// accounts, the authority is initialized to the stake state account's withdrawer.
    #[account(
        0,
        name = "config",
        desc = "Stake config account"
    )]
    #[account(
        1,
        writable,
        name = "sol_staker_stake",
        desc = "SOL staker stake account (pda of `['stake::state::sol_staker_stake', stake state, config]`)"
    )]
    #[account(
        2,
        writable,
        name = "validator_stake",
        desc = "Validator stake account (pda of `['stake::state::validator_stake', validator, config]`)"
    )]
    #[account(
        3,
        name = "sol_staker_native_stake",
        desc = "Sol staker native stake"
    )]
    #[account(
        4,
        name = "sysvar_stake_history",
        desc = "Sysvar stake history"
    )]
    #[account(
        5,
        name = "system_program",
        desc = "System program"
    )]
    #[account(
        6,
        name = "sol_stake_view_program",
        desc = "Paladin SOL Stake View program"
    )]
    InitializeSolStakerStake,

    /// Stakes tokens with the given config.
    ///
    /// NOTE: This instruction is used by SOL staker stake accounts. The total amount of staked
    /// tokens is limited to the 1.3 * current amount of SOL staked by the SOL staker.
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
        name = "sol_staker_stake",
        desc = "SOL staker stake account (pda of `['stake::state::sol_staker_stake', stake state, config]`)"
    )]
    #[account(
        2,
        writable,
        name = "sol_staker_stake_authority",
        desc = "SOL staker stake authority account"
    )]
    #[account(
        3,
        writable,
        name = "source_token_account",
        desc = "Token account"
    )]
    #[account(
        4,
        signer,
        name = "token_account_authority",
        desc = "Owner or delegate of the token account"
    )]
    #[account(
        5,
        name = "mint",
        desc = "Stake Token Mint"
    )]
    #[account(
        6,
        writable,
        name = "vault",
        desc = "Stake token Vault"
    )]
    #[account(
        7,
        name = "vault_holder_rewards",
        desc = "Stake token Vault"
    )]
    #[account(
        8,
        name = "token_program",
        desc = "Token program"
    )]
    SolStakerStakeTokens(u64),

    /// Harvests stake SOL rewards earned by the given sol staker stake account.
    ///
    /// NOTE: This is very similar to the logic in the rewards program. Since the
    /// staking rewards are held in a separate account, they must be distributed
    /// based on the proportion of total stake.
    #[account(
        0,
        name = "sol_stake_view_program",
        desc = "Sol stake view program"
    )]
    #[account(
        1,
        writable,
        name = "config",
        desc = "Stake config account"
    )]
    #[account(
        2,
        name = "vault_holder_rewards",
        desc = "Holder rewards account"
    )]
    #[account(
        3,
        writable,
        name = "sol_staker_stake",
        desc = "SOL staker stake account (pda of `['stake::state::sol_staker_stake', stake state, config]`)"
    )]
    #[account(
        4,
        writable,
        name = "sol_staker_stake_authority",
        desc = "SOL staker stake authority"
    )]
    #[account(
        5,
        name = "sol_staker_native_stake",
        desc = "Native stake account"
    )]
    #[account(
        6,
        writable,
        name = "validator_stake",
        desc = "Validator stake account"
    )]
    #[account(
        7,
        writable,
        name = "validator_stake_authority",
        desc = "Validator stake authority"
    )]
    #[account(
        8,
        name = "sysvar_stake_history",
        desc = "Stake history sysvar"
    )]
    #[account(
        9,
        optional,
        writable,
        name = "keeper_recipient",
        desc = "Recipient for sol sync bounty"
    )]
    HarvestSolStakerRewards,

    /// Move tokens from deactivating to inactive.
    ///
    /// Reduces the total voting power for the SOL staker stake account and the total staked
    /// amount on the correspondent validator stake and config accounts.
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
        name = "sol_staker_stake",
        desc = "SOL staker stake account (pda of `['stake::state::sol_staker_stake', stake state, config]`)"
    )]
    #[account(
        2,
        writable,
        name = "sol_staker_stake_authority",
        desc = "SOL staker stake authority account"
    )]
    #[account(
        3,
        writable,
        name = "vault_holder_rewards",
        desc = "Vault holder rewards account"
    )]
    InactivateSolStakerStake,

    /// Slashes a validator stake account for the given amount.
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
        name = "sol_staker_stake",
        desc = "SOL staker stake account (pda of `['stake::state::sol_staker_stake', stake state, config]`)"
    )]
    #[account(
        2,
        writable,
        name = "sol_staker_stake_authority",
        desc = "SOL staker stake authority account"
    )]
    #[account(
        3,
        signer,
        name = "slash_authority",
        desc = "Config slash authority"
    )]
    #[account(
        4,
        writable,
        name = "mint",
        desc = "Vault token mint"
    )]
    #[account(
        5,
        writable,
        name = "vault",
        desc = "Vault token account"
    )]
    #[account(
        6,
        name = "vault_holder_rewards",
        desc = "Vault holder rewards account"
    )]
    #[account(
        7,
        name = "vault_authority",
        desc = "Vault authority (pda of `['token-owner', config]`)"
    )]
    #[account(
        8,
        name = "token_program",
        desc = "Token program"
    )]
    SlashSolStakerStake(u64),
}

impl StakeInstruction {
    /// Packs a [StakeInstruction](enum.StakeInstruction.html) into a byte buffer.
    pub fn pack(&self) -> Vec<u8> {
        match self {
            StakeInstruction::InitializeConfig {
                slash_authority,
                config_authority,
                cooldown_time_seconds,
                max_deactivation_basis_points,
                sync_rewards_lamports,
            } => {
                let mut data = Vec::with_capacity(11);
                data.push(0);
                data.extend_from_slice(&slash_authority.to_bytes());
                data.extend_from_slice(&config_authority.to_bytes());
                data.extend_from_slice(&cooldown_time_seconds.to_le_bytes());
                data.extend_from_slice(&max_deactivation_basis_points.to_le_bytes());
                data.extend_from_slice(&sync_rewards_lamports.to_le_bytes());
                data
            }
            StakeInstruction::InitializeValidatorStake => vec![1],
            StakeInstruction::ValidatorStakeTokens(amount) => {
                let mut data = Vec::with_capacity(9);
                data.push(2);
                data.extend_from_slice(&amount.to_le_bytes());
                data
            }
            StakeInstruction::DeactivateStake(amount) => {
                let mut data = Vec::with_capacity(9);
                data.push(3);
                data.extend_from_slice(&amount.to_le_bytes());
                data
            }
            StakeInstruction::InactivateValidatorStake => vec![4],
            StakeInstruction::WithdrawInactiveStake(amount) => {
                let mut data = Vec::with_capacity(9);
                data.push(5);
                data.extend_from_slice(&amount.to_le_bytes());
                data
            }
            StakeInstruction::HarvestHolderRewards => vec![6],
            StakeInstruction::HarvestValidatorRewards => vec![7],
            StakeInstruction::SlashValidatorStake(amount) => {
                let mut data = Vec::with_capacity(9);
                data.push(8);
                data.extend_from_slice(&amount.to_le_bytes());
                data
            }
            StakeInstruction::SetAuthority(authority_type) => {
                vec![
                    9,
                    match authority_type {
                        AuthorityType::Config => 0,
                        AuthorityType::Slash => 1,
                        AuthorityType::Stake => 2,
                    },
                ]
            }
            StakeInstruction::UpdateConfig(field) => {
                let mut data = Vec::with_capacity(11);
                data.push(10);
                match field {
                    ConfigField::CooldownTimeSeconds(value) => {
                        data.push(0);
                        data.extend_from_slice(&value.to_le_bytes());
                    }
                    ConfigField::MaxDeactivationBasisPoints(value) => {
                        data.push(1);
                        data.extend_from_slice(&value.to_le_bytes());
                    }
                    ConfigField::SyncRewardsLamports(value) => {
                        data.push(2);
                        data.extend_from_slice(&value.to_le_bytes());
                    }
                }
                data
            }
            StakeInstruction::InitializeSolStakerStake => vec![11],
            StakeInstruction::SolStakerStakeTokens(amount) => {
                let mut data = Vec::with_capacity(9);
                data.push(12);
                data.extend_from_slice(&amount.to_le_bytes());
                data
            }
            StakeInstruction::HarvestSolStakerRewards => vec![13],
            StakeInstruction::InactivateSolStakerStake => vec![14],
            StakeInstruction::SlashSolStakerStake(amount) => {
                let mut data = Vec::with_capacity(9);
                data.push(15);
                data.extend_from_slice(&amount.to_le_bytes());
                data
            }
        }
    }

    /// Unpacks a byte buffer into a [StakeInstruction](enum.StakeInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        match input.split_first() {
            // 0 - InitializeConfig: u64 (8) + u16 (2) + u64 (8)
            Some((&0, rest)) if rest.len() == 32 + 32 + 8 + 2 + 8 => {
                let slash_authority = Pubkey::new_from_array(*array_ref![rest, 0, 32]);
                let config_authority = Pubkey::new_from_array(*array_ref![rest, 32, 32]);
                let cooldown_time_seconds = u64::from_le_bytes(*array_ref![rest, 64, 8]);
                let max_deactivation_basis_points = u16::from_le_bytes(*array_ref![rest, 72, 2]);
                let sync_rewards_lamports = u64::from_le_bytes(*array_ref![rest, 74, 8]);

                Ok(StakeInstruction::InitializeConfig {
                    slash_authority,
                    config_authority,
                    cooldown_time_seconds,
                    max_deactivation_basis_points,
                    sync_rewards_lamports,
                })
            }
            // 1 - InitializeValidatorStake
            Some((&1, _)) => Ok(StakeInstruction::InitializeValidatorStake),
            // 2 - StakeTokens: u64 (8)
            Some((&2, rest)) if rest.len() == 8 => {
                let amount = u64::from_le_bytes(*array_ref![rest, 0, 8]);

                Ok(StakeInstruction::ValidatorStakeTokens(amount))
            }
            // 3 - DeactivateStake: u64 (8)
            Some((&3, rest)) if rest.len() == 8 => {
                let amount = u64::from_le_bytes(*array_ref![rest, 0, 8]);

                Ok(StakeInstruction::DeactivateStake(amount))
            }
            // 4 - InactivateValidatorStake
            Some((&4, _)) => Ok(StakeInstruction::InactivateValidatorStake),
            // 5 - WithdrawInactiveStake: u64 (8)
            Some((&5, rest)) if rest.len() == 8 => {
                let amount = u64::from_le_bytes(*array_ref![rest, 0, 8]);

                Ok(StakeInstruction::WithdrawInactiveStake(amount))
            }
            // 6 - HarvestHolderRewards
            Some((&6, _)) => Ok(StakeInstruction::HarvestHolderRewards),
            // 7 - HarvestStakeRewards
            Some((&7, _)) => Ok(StakeInstruction::HarvestValidatorRewards),
            // 8 - SlashValidatorStake: u64 (8)
            Some((&8, rest)) if rest.len() == 8 => {
                let amount = u64::from_le_bytes(*array_ref![rest, 0, 8]);

                Ok(StakeInstruction::SlashValidatorStake(amount))
            }
            // 9 - SetAuthority: AuthorityType (u8))
            Some((&9, rest)) if rest.len() == 1 => {
                let authority_type =
                    FromPrimitive::from_u8(rest[0]).ok_or(ProgramError::InvalidInstructionData)?;
                Ok(StakeInstruction::SetAuthority(authority_type))
            }
            // 10 - UpdateConfig: ConfigField (u64 or u16)
            Some((&10, rest)) => {
                let field = match rest.split_first() {
                    Some((&0, rest)) if rest.len() == 8 => {
                        ConfigField::CooldownTimeSeconds(u64::from_le_bytes(*array_ref![
                            rest, 0, 8
                        ]))
                    }
                    Some((&1, rest)) if rest.len() == 2 => {
                        ConfigField::MaxDeactivationBasisPoints(u16::from_le_bytes(*array_ref![
                            rest, 0, 2
                        ]))
                    }
                    Some((&2, rest)) if rest.len() == 8 => {
                        ConfigField::SyncRewardsLamports(u64::from_le_bytes(*array_ref![
                            rest, 0, 8
                        ]))
                    }
                    _ => return Err(ProgramError::InvalidInstructionData),
                };

                Ok(StakeInstruction::UpdateConfig(field))
            }
            // 11 - InitializeSolStakerStake
            Some((&11, _)) => Ok(StakeInstruction::InitializeSolStakerStake),
            // 12 - SolStakerStakeTokens: u64 (8)
            Some((&12, rest)) if rest.len() == 8 => {
                let amount = u64::from_le_bytes(*array_ref![rest, 0, 8]);

                Ok(StakeInstruction::SolStakerStakeTokens(amount))
            }
            // 13 - HarvestSolStakerRewards
            Some((&13, _)) => Ok(StakeInstruction::HarvestSolStakerRewards),
            // 14 - InactivateSolStakerStake
            Some((&14, _)) => Ok(StakeInstruction::InactivateSolStakerStake),
            // 15 - SlashSolStakerStake: u64 (8)
            Some((&15, rest)) if rest.len() == 8 => {
                let amount = u64::from_le_bytes(*array_ref![rest, 0, 8]);

                Ok(StakeInstruction::SlashSolStakerStake(amount))
            }
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}

/// Enum defining all authorities in the program
#[derive(Clone, Debug, Eq, FromPrimitive, PartialEq, ShankType)]
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
    /// Lamports amount paid to for syncing a SOL stake account.
    SyncRewardsLamports(u64),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_unpack_initialize_config() {
        let original = StakeInstruction::InitializeConfig {
            slash_authority: Pubkey::new_unique(),
            config_authority: Pubkey::new_unique(),
            cooldown_time_seconds: 120,
            max_deactivation_basis_points: 500,
            sync_rewards_lamports: 100,
        };
        let packed = original.pack();
        let unpacked = StakeInstruction::unpack(&packed).unwrap();
        assert_eq!(original, unpacked);
    }

    #[test]
    fn test_pack_unpack_initialize_stake() {
        let original = StakeInstruction::InitializeValidatorStake;
        let packed = original.pack();
        let unpacked = StakeInstruction::unpack(&packed).unwrap();
        assert_eq!(original, unpacked);
    }

    #[test]
    fn test_pack_unpack_stake_tokens() {
        let original = StakeInstruction::ValidatorStakeTokens(100);
        let packed = original.pack();
        let unpacked = StakeInstruction::unpack(&packed).unwrap();
        assert_eq!(original, unpacked);
    }

    #[test]
    fn test_pack_unpack_deactivate_stake() {
        let original = StakeInstruction::DeactivateStake(100);
        let packed = original.pack();
        let unpacked = StakeInstruction::unpack(&packed).unwrap();
        assert_eq!(original, unpacked);
    }

    #[test]
    fn test_pack_unpack_inactivate_stake() {
        let original = StakeInstruction::InactivateValidatorStake;
        let packed = original.pack();
        let unpacked = StakeInstruction::unpack(&packed).unwrap();
        assert_eq!(original, unpacked);
    }

    #[test]
    fn test_pack_unpack_withdraw_inactive_stake() {
        let original = StakeInstruction::WithdrawInactiveStake(100);
        let packed = original.pack();
        let unpacked = StakeInstruction::unpack(&packed).unwrap();
        assert_eq!(original, unpacked);
    }

    #[test]
    fn test_pack_unpack_harvest_holder_rewards() {
        let original = StakeInstruction::HarvestHolderRewards;
        let packed = original.pack();
        let unpacked = StakeInstruction::unpack(&packed).unwrap();
        assert_eq!(original, unpacked);
    }

    #[test]
    fn test_pack_unpack_harvest_stake_rewards() {
        let original = StakeInstruction::HarvestValidatorRewards;
        let packed = original.pack();
        let unpacked = StakeInstruction::unpack(&packed).unwrap();
        assert_eq!(original, unpacked);
    }

    #[test]
    fn test_pack_unpack_slash_validator_stake() {
        let original = StakeInstruction::SlashValidatorStake(100);
        let packed = original.pack();
        let unpacked = StakeInstruction::unpack(&packed).unwrap();
        assert_eq!(original, unpacked);
    }

    #[test]
    fn test_pack_unpack_set_authority() {
        let original = StakeInstruction::SetAuthority(AuthorityType::Config);
        let packed = original.pack();
        let unpacked = StakeInstruction::unpack(&packed).unwrap();
        assert_eq!(original, unpacked);
    }

    #[test]
    fn test_pack_unpack_update_config() {
        let original = StakeInstruction::UpdateConfig(ConfigField::CooldownTimeSeconds(120));
        let packed = original.pack();
        let unpacked = StakeInstruction::unpack(&packed).unwrap();
        assert_eq!(original, unpacked);

        let original = StakeInstruction::UpdateConfig(ConfigField::MaxDeactivationBasisPoints(500));
        let packed = original.pack();
        let unpacked = StakeInstruction::unpack(&packed).unwrap();
        assert_eq!(original, unpacked);
    }
}
