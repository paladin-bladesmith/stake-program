use paladin_rewards_program_client::accounts::HolderRewards;
use solana_program::{entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey};

use crate::{
    error::StakeError,
    instruction::accounts::{Context, HarvestValidatorRewardsAccounts},
    processor::{harvest, unpack_initialized, unpack_initialized_mut, HarvestAccounts},
    require,
    state::{find_validator_stake_pda, Config, ValidatorStake},
};

/// Harvests stake SOL rewards earned by the given validator stake account.
///
/// NOTE: This is very similar to the logic in the rewards program. Since the
/// staking rewards are held in a separate account, they must be distributed
/// based on the proportion of total stake.
///
/// 0. `[w]` Config account
/// 1. `[ ]` Vault holder rewards
/// 2. `[w]` Validator stake
/// 3. `[w]` Validator stake authority
pub fn process_harvest_validator_rewards(
    program_id: &Pubkey,
    ctx: Context<HarvestValidatorRewardsAccounts>,
) -> ProgramResult {
    // Account validation.

    // config
    // - owner must be the stake program
    // - must be initialized
    require!(
        ctx.accounts.config.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "config"
    );
    let vault_key = {
        let config = ctx.accounts.config.data.borrow();
        let config = unpack_initialized::<Config>(&config)?;

        config.vault
    };

    // stake
    // - owner must be the stake program
    // - must be initialized
    // - derivation must match (validates the config account)
    require!(
        ctx.accounts.validator_stake.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "validator stake"
    );

    let mut stake_data = ctx.accounts.validator_stake.try_borrow_mut_data()?;
    let validator_stake = unpack_initialized_mut::<ValidatorStake>(&mut stake_data)?;

    let (derivation, _) = find_validator_stake_pda(
        &validator_stake.delegation.validator_vote,
        ctx.accounts.config.key,
        program_id,
    );

    require!(
        ctx.accounts.validator_stake.key == &derivation,
        ProgramError::InvalidSeeds,
        "stake",
    );

    // stake authority
    // - must match the authority on the stake account
    require!(
        ctx.accounts.validator_stake_authority.key == &validator_stake.delegation.authority,
        StakeError::InvalidAuthority,
        "stake authority",
    );

    // Holder rewards.
    // - Must be derived from the vault account.
    let derivation = HolderRewards::find_pda(&vault_key).0;
    require!(
        ctx.accounts.vault_holder_rewards.key == &derivation,
        ProgramError::InvalidSeeds,
        "vault_holder_rewards"
    );

    // Process the harvest.
    harvest(
        program_id,
        HarvestAccounts {
            config: ctx.accounts.config,
            vault_holder_rewards: ctx.accounts.vault_holder_rewards,
            authority: ctx.accounts.validator_stake_authority,
        },
        &mut validator_stake.delegation,
        None,
    )?;

    Ok(())
}
