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
    processor::{harvest, sync_effective, unpack_initialized_mut, HarvestAccounts},
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
/// 0. `[ ]` Paladin SOL stake view program
/// 1. `[w]` Config account
/// 2. `[w]` Vault holder rewards
/// 3. `[w]` Sol staker stake
/// 4. `[w]` Sol staker stake authority
/// 5. `[ ]` Sol staker native stake
/// 6. `[w]` Validator stake
/// 7. `[w]` Validator stake authority
/// 8. `[ ]` Sysvar stake history
/// 9. `[w]?` Keeper recipient
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
    let mut config = ctx.accounts.config.data.borrow_mut();
    let config = unpack_initialized_mut::<Config>(&mut config)?;

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
        ctx.accounts.sol_staker_native_stake.key == &sol_staker_stake.sol_stake,
        StakeError::IncorrectSolStakeAccount,
        "sol stake"
    );

    // Previous validator.
    // - owner must be the stake program
    // - must have the correct derivation (validates both the validator vote
    //   and config accounts)
    // - must be initialized
    // - must belong to the validator the staker was previously delegated to
    require!(
        ctx.accounts.previous_validator_stake.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "validator stake"
    );
    let (derivation, _) = find_validator_stake_pda(
        &sol_staker_stake.delegation.validator_vote,
        ctx.accounts.config.key,
        program_id,
    );
    require!(
        ctx.accounts.previous_validator_stake.key == &derivation,
        ProgramError::InvalidSeeds,
        "previous validator stake",
    );
    let mut previous_validator_data = ctx
        .accounts
        .previous_validator_stake
        .try_borrow_mut_data()?;
    let previous_validator_stake =
        unpack_initialized_mut::<ValidatorStake>(&mut previous_validator_data)?;

    // Holder rewards.
    // - Must be derived from the vault account.
    let derivation = HolderRewards::find_pda(&config.vault).0;
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
        .stake(ctx.accounts.sol_staker_native_stake)
        .stake_history(ctx.accounts.sysvar_stake_history)
        .invoke()?;
    let (_, return_data) = get_return_data().ok_or(ProgramError::InvalidAccountData)?;
    let stake_state_data =
        bytemuck::try_from_bytes::<GetStakeActivatingAndDeactivatingReturnData>(&return_data)
            .map_err(|_error| ProgramError::InvalidAccountData)?;
    let current_delegation = stake_state_data.delegated_vote.get();
    let mut stake_amount = stake_state_data.effective.into();
    let requires_sync = stake_amount != sol_staker_stake.lamports_amount
        || current_delegation != Some(sol_staker_stake.delegation.validator_vote);

    // Harvest the staker.
    harvest(
        HarvestAccounts {
            config: ctx.accounts.config,
            vault_holder_rewards: ctx.accounts.vault_holder_rewards,
            authority: ctx.accounts.sol_staker_stake_authority,
        },
        config,
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

    // If no sync is required, then we are done.
    if !requires_sync {
        return Ok(());
    }

    // If the user has a previous delegation, their old stake is removed.
    if sol_staker_stake.delegation.validator_vote != Pubkey::default() {
        // Harvest the previous validator to flush rewards before we update their stake.
        harvest(
            HarvestAccounts {
                config: ctx.accounts.config,
                vault_holder_rewards: ctx.accounts.vault_holder_rewards,
                authority: ctx.accounts.previous_validator_stake_authority,
            },
            config,
            &mut previous_validator_stake.delegation,
            None,
        )?;

        // Remove the staker's old SOL amount from the previous validator total.
        previous_validator_stake.total_staked_lamports_amount = previous_validator_stake
            .total_staked_lamports_amount
            .checked_sub(sol_staker_stake.lamports_amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        // Update the validator's effective stake.
        sync_effective(
            config,
            &mut previous_validator_stake.delegation,
            previous_validator_stake.total_staked_lamports_amount,
        )?;
    } else {
        assert_eq!(sol_staker_stake.lamports_amount, 0);
    }

    // If the user has a current delegation, their new stake is added here.
    if let Some(current_delegation) = current_delegation {
        // Current validator.
        // - owner must be the stake program
        // - must have the correct derivation (validates both the validator vote
        //   and config accounts)
        // - must be initialized
        // - must belong to the validator the staker is currently delegated to
        let (derivation, _) =
            find_validator_stake_pda(&current_delegation, ctx.accounts.config.key, program_id);
        require!(
            ctx.accounts.current_validator_stake.key == &derivation,
            ProgramError::InvalidSeeds,
            "current validator stake",
        );

        // Only credit the stake to the new validator if it's a paladin-enabled validator.
        if ctx.accounts.current_validator_stake.owner == program_id {
            let mut current_validator_data =
                ctx.accounts.current_validator_stake.try_borrow_mut_data()?;
            let current_validator_stake =
                unpack_initialized_mut::<ValidatorStake>(&mut current_validator_data)?;

            // Harvest the current validator.
            harvest(
                HarvestAccounts {
                    config: ctx.accounts.config,
                    vault_holder_rewards: ctx.accounts.vault_holder_rewards,
                    authority: ctx.accounts.current_validator_stake_authority,
                },
                config,
                &mut current_validator_stake.delegation,
                None,
            )?;

            // Add the user's stake to the current validator.
            current_validator_stake.total_staked_lamports_amount = current_validator_stake
                .total_staked_lamports_amount
                .checked_add(stake_amount)
                .ok_or(ProgramError::ArithmeticOverflow)?;
        } else {
            stake_amount = 0;
        }
    }

    // Finally, the user's stake is updated.
    sol_staker_stake.lamports_amount = stake_amount;

    Ok(())
}
