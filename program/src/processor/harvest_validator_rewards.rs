use solana_program::{entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey};

use crate::{
    error::StakeError,
    instruction::accounts::{Context, HarvestValidatorRewardsAccounts},
    processor::{harvest, unpack_initialized, unpack_initialized_mut},
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
/// 1. `[w]` Validator stake account
/// 2. `[w]` Stake authority
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

    let config_data = ctx.accounts.config.try_borrow_data()?;
    let config = unpack_initialized::<Config>(&config_data)?;

    require!(
        config.is_initialized(),
        ProgramError::UninitializedAccount,
        "config",
    );

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
        ctx.accounts.stake_authority.key == &validator_stake.delegation.authority,
        StakeError::InvalidAuthority,
        "stake authority",
    );

    // Process the harvest.
    harvest(
        (config, ctx.accounts.config),
        &mut validator_stake.delegation,
        ctx.accounts.stake_authority,
        None,
    )?;

    Ok(())
}
