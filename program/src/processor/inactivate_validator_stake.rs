use solana_program::{
    clock::Clock, entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey,
    sysvar::Sysvar,
};
use spl_token_2022::{
    extension::PodStateWithExtensions,
    pod::{PodAccount, PodMint},
};

use crate::{
    error::StakeError,
    instruction::accounts::{Context, InactivateValidatorStakeAccounts},
    processor::{harvest, sync_effective, unpack_initialized_mut, HarvestAccounts},
    require,
    state::{
        create_vault_pda, find_validator_stake_pda, get_vault_pda_signer_seeds, Config,
        ValidatorStake,
    },
};

/// Move tokens from deactivating to inactive.
///
/// Reduces the total voting power for the validator stake account and the total staked
/// amount on the system.
///
/// NOTE: This instruction is permissionless, so anybody can finish
/// deactivating validator's tokens, preparing them to be withdrawn.
///
/// 0. `[w]` Stake config account
/// 1. `[w]` Validator stake account
pub fn process_inactivate_validator_stake<'info>(
    program_id: &Pubkey,
    ctx: Context<'info, InactivateValidatorStakeAccounts<'info>>,
    amount: u64,
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

    // vault
    // - must be the token account on the stake config account
    require!(
        ctx.accounts.vault.key == &config.vault,
        StakeError::IncorrectVaultAccount,
    );
    require!(
        ctx.accounts.vault.key != ctx.accounts.destination_token_account.key,
        StakeError::InvalidDestinationAccount,
        "vault matches destination token account"
    );
    let vault_data = ctx.accounts.vault.try_borrow_data()?;
    let vault = PodStateWithExtensions::<PodAccount>::unpack(&vault_data)?;

    // vault authority
    // - derivation must match
    let bump = [config.vault_authority_bump];
    let vault_signer = create_vault_pda(ctx.accounts.config.key, &bump, program_id)?;
    require!(
        ctx.accounts.vault_authority.key == &vault_signer,
        StakeError::InvalidAuthority,
        "vault authority",
    );
    let signer_seeds = get_vault_pda_signer_seeds(ctx.accounts.config.key, &bump);

    // stake
    // - owner must be the stake program
    // - must be a ValidatorStake account
    // - must be initialized
    // - must have the correct derivation
    require!(
        ctx.accounts.validator_stake.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "stake"
    );
    let stake_data = &mut ctx.accounts.validator_stake.try_borrow_mut_data()?;
    let validator_stake = unpack_initialized_mut::<ValidatorStake>(stake_data)?;
    let (derivation, _) = find_validator_stake_pda(
        &validator_stake.delegation.validator_vote,
        ctx.accounts.config.key,
        program_id,
    );
    require!(
        ctx.accounts.validator_stake.key == &derivation,
        ProgramError::InvalidSeeds,
        "validator stake",
    );
    let delegation = &mut validator_stake.delegation;

    // authority
    // - must be a signer
    // - must match the authority on the stake account
    require!(
        ctx.accounts.validator_stake_authority.is_signer,
        ProgramError::MissingRequiredSignature,
        "stake_authority"
    );
    require!(
        ctx.accounts.validator_stake_authority.key == &delegation.authority,
        StakeError::InvalidAuthority,
        "stake_authority"
    );

    // mint
    // - must match the stake vault mint
    require!(
        &vault.base.mint == ctx.accounts.mint.key,
        StakeError::InvalidMint,
        "mint"
    );
    let mint_data = ctx.accounts.mint.try_borrow_data()?;
    let mint = PodStateWithExtensions::<PodMint>::unpack(&mint_data)?;
    let decimals = mint.base.decimals;

    // Harvest rewards & update last claim tracking.
    harvest(
        HarvestAccounts {
            config: ctx.accounts.config,
            vault_holder_rewards: ctx.accounts.vault_holder_rewards,
            authority: ctx.accounts.validator_stake_authority,
        },
        config,
        delegation,
        None,
    )?;

    // Ensure we are not in a cooldown period.
    let now = Clock::get()?.unix_timestamp as u64;
    require!(
        delegation.unstake_cooldown < now,
        StakeError::ActiveUnstakeCooldown,
    );

    // Update staked amount & unstake cooldown.
    let staked_amount = delegation
        .staked_amount
        .checked_sub(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    delegation.staked_amount = staked_amount;
    delegation.unstake_cooldown = now;

    // TODO:
    //
    // - Check the stake cooldown.
    // - Remove the staked amount.
    // - Transfer the tokens to the staker authority.
    // - Set the stake cooldown.
    // - Sync effective.

    sync_effective(
        config,
        delegation,
        (
            validator_stake.total_staked_lamports_amount,
            validator_stake.total_staked_lamports_amount_min,
        ),
    )?;

    spl_token_2022::onchain::invoke_transfer_checked(
        &spl_token_2022::ID,
        ctx.accounts.vault.clone(),
        ctx.accounts.mint.clone(),
        ctx.accounts.destination_token_account.clone(),
        ctx.accounts.vault_authority.clone(),
        ctx.remaining_accounts,
        amount,
        decimals,
        &[&signer_seeds],
    );

    Ok(())
}
