use paladin_rewards_program_client::accounts::HolderRewards;
use paladin_sol_stake_view_program_client::{
    instructions::GetStakeActivatingAndDeactivatingCpiBuilder,
    GetStakeActivatingAndDeactivatingReturnData,
};
use solana_program::{
    entrypoint::ProgramResult, program::get_return_data, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    error::StakeError,
    instruction::accounts::{Context, HarvestSolStakerRewardsAccounts},
    processor::{harvest, unpack_initialized, unpack_initialized_mut, HarvestAccounts},
    require,
    state::{
        find_sol_staker_stake_pda, find_validator_stake_pda, Config, SolStakerStake, ValidatorStake,
    },
};

/// Harvests stake SOL rewards earned by the given SOL staker stake account.
///
/// NOTE: This is very similar to the logic in the rewards program. Since the
/// staking rewards are held in a separate account, they must be distributed
/// based on the proportion of total stake.
///
/// 0. `[w]` Config account
/// 1. `[w]` Vault authority
/// 2. `[w]` Staker PAL stake account
/// 3. `[w]` Staker authority
/// 4. `[ ]` Staker native stake account
/// 5. `[w]` Validator stake account
/// 6. `[w]` Validator stake authority
/// 7. `[ ]` Stake history sysvar
/// 8. `[ ]` Paladin SOL stake view program
pub fn process_harvest_sol_staker_rewards(
    program_id: &Pubkey,
    ctx: Context<HarvestSolStakerRewardsAccounts>,
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

    // sol staker stake
    // - owner must be the stake program
    // - must be initialized
    // - derivation must match (validates the config account)
    require!(
        ctx.accounts.sol_staker_stake.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "sol staker stake"
    );

    let mut sol_staker_stake_data = ctx.accounts.sol_staker_stake.try_borrow_mut_data()?;
    let sol_staker_stake = unpack_initialized_mut::<SolStakerStake>(&mut sol_staker_stake_data)?;
    let (derivation, _) = find_sol_staker_stake_pda(
        &sol_staker_stake.sol_stake,
        ctx.accounts.config.key,
        program_id,
    );
    require!(
        ctx.accounts.sol_staker_stake.key == &derivation,
        ProgramError::InvalidSeeds,
        "sol stake stake",
    );

    // stake authority
    // - must match the authority on the stake account
    require!(
        ctx.accounts.sol_staker_stake_authority.key == &sol_staker_stake.delegation.authority,
        StakeError::InvalidAuthority,
        "stake authority",
    );

    // Native stake.
    // - Must match the PAL staker specified stake account.
    require!(
        ctx.accounts.native_stake.key == &sol_staker_stake.sol_stake,
        StakeError::IncorrectSolStakeAccount,
        "sol stake"
    );

    // validator stake
    // - owner must be the stake program
    // - must have the correct derivation (validates both the validator vote
    //   and config accounts)
    // - must be initialized
    require!(
        ctx.accounts.validator_stake.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "validator stake"
    );

    // validator vote must match the SOL staker stake state account's validator vote
    // (validation done on the derivation of the expected address)
    let (derivation, _) = find_validator_stake_pda(
        &sol_staker_stake.delegation.validator_vote,
        ctx.accounts.config.key,
        program_id,
    );
    require!(
        ctx.accounts.validator_stake.key == &derivation,
        ProgramError::InvalidSeeds,
        "validator stake",
    );
    let mut stake_data = ctx.accounts.validator_stake.try_borrow_mut_data()?;
    let validator_stake = unpack_initialized_mut::<ValidatorStake>(&mut stake_data)?;

    // Holder rewards.
    // - Must be derived from the vault account.
    let derivation = HolderRewards::find_pda(&vault_key).0;
    require!(
        ctx.accounts.vault_holder_rewards.key == &derivation,
        ProgramError::InvalidSeeds,
        "vault_holder_rewards"
    );

    // Sol stake view program.
    // - Must match the expected program ID.
    require!(
        ctx.accounts.sol_stake_view_program.key == &paladin_sol_stake_view_program_client::ID,
        ProgramError::IncorrectProgramId,
        "invalid sol stake view program"
    );

    // Compute the latest native stake for this staker.
    GetStakeActivatingAndDeactivatingCpiBuilder::new(ctx.accounts.sol_stake_view_program)
        .stake(ctx.accounts.native_stake)
        .stake_history(ctx.accounts.sysvar_stake_history)
        .invoke()?;
    let (_, return_data) = get_return_data().ok_or(ProgramError::InvalidAccountData)?;
    let stake_state_data =
        bytemuck::try_from_bytes::<GetStakeActivatingAndDeactivatingReturnData>(&return_data)
            .map_err(|_error| ProgramError::InvalidAccountData)?;
    let delegated_vote = stake_state_data.delegated_vote.get();
    let delegate_changed = delegated_vote != Some(sol_staker_stake.delegation.validator_vote);
    let stake_amount = match delegate_changed {
        // TODO: We zero their effective PAL, but how do they re-activate it?
        true => 0,
        false => u64::from(stake_state_data.activating)
            .checked_add(stake_state_data.effective.into())
            .and_then(|amount| amount.checked_sub(u64::from(stake_state_data.deactivating)))
            .ok_or(ProgramError::ArithmeticOverflow)?,
    };
    let requires_sync = stake_amount != sol_staker_stake.lamports_amount;

    // Harvest the staker.
    harvest(
        HarvestAccounts {
            config: ctx.accounts.config,
            holder_rewards: ctx.accounts.vault_holder_rewards,
            recipient: ctx.accounts.sol_staker_stake_authority,
        },
        &mut sol_staker_stake.delegation,
        match requires_sync {
            true => Some(
                ctx.accounts
                    .keeper_recipient
                    .ok_or(ProgramError::NotEnoughAccountKeys)?,
            ),
            false => None,
        },
    )?;

    if requires_sync {
        // Harvest the validator.
        harvest(
            HarvestAccounts {
                config: ctx.accounts.config,
                holder_rewards: ctx.accounts.vault_holder_rewards,
                recipient: ctx.accounts.validator_stake_authority,
            },
            &mut validator_stake.delegation,
            None,
        )?;

        // Remove the staker's old SOL amount from the validator total.
        validator_stake.total_staked_lamports_amount = validator_stake
            .total_staked_lamports_amount
            .checked_sub(sol_staker_stake.lamports_amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        // Update the stakers's SOL amount.
        sol_staker_stake.lamports_amount = stake_amount;

        // Add the staker's new SOL amount to the validator total.
        validator_stake.total_staked_lamports_amount = validator_stake
            .total_staked_lamports_amount
            .checked_add(stake_amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;
    }

    Ok(())
}
