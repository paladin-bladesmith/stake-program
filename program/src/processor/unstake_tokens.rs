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
    instruction::accounts::{Context, UnstakeTokensAccounts},
    processor::{harvest, sync_effective, unpack_initialized_mut, HarvestAccounts},
    require,
    state::{
        create_vault_pda, find_sol_staker_stake_pda, find_validator_stake_pda,
        get_vault_pda_signer_seeds, Config, SolStakerStake, ValidatorStake, MAX_BASIS_POINTS,
    },
};

pub fn process_unstake_tokens<'info>(
    program_id: &Pubkey,
    ctx: Context<'info, UnstakeTokensAccounts<'info>>,
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
    let mut config_borrow = ctx.accounts.config.data.borrow_mut();
    let config = unpack_initialized_mut::<Config>(&mut config_borrow)?;

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
    let vault_borrow = ctx.accounts.vault.try_borrow_data()?;
    let vault = PodStateWithExtensions::<PodAccount>::unpack(&vault_borrow)?;

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
        ctx.accounts.stake.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "stake"
    );
    let stake_borrow = &mut ctx.accounts.stake.try_borrow_mut_data()?;
    let (derivation, lamports, lamports_min, delegation) = match stake_borrow.len() {
        ValidatorStake::LEN => {
            let stake = unpack_initialized_mut::<ValidatorStake>(stake_borrow)?;

            (
                find_validator_stake_pda(
                    &stake.delegation.validator_vote,
                    ctx.accounts.config.key,
                    program_id,
                )
                .0,
                stake.total_staked_lamports_amount,
                stake.total_staked_lamports_amount_min,
                &mut stake.delegation,
            )
        }
        SolStakerStake::LEN => {
            let stake = unpack_initialized_mut::<SolStakerStake>(stake_borrow)?;

            (
                find_sol_staker_stake_pda(&stake.sol_stake, ctx.accounts.config.key, program_id).0,
                stake.lamports_amount,
                0,
                &mut stake.delegation,
            )
        }
        _ => return Err(ProgramError::InvalidAccountData),
    };
    require!(
        ctx.accounts.stake.key == &derivation,
        ProgramError::InvalidSeeds,
        "validator stake",
    );

    // authority
    // - must be a signer
    // - must match the authority on the stake account
    require!(
        ctx.accounts.stake_authority.is_signer,
        ProgramError::MissingRequiredSignature,
        "stake_authority"
    );
    require!(
        ctx.accounts.stake_authority.key == &delegation.authority,
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
    let mint_borrow = ctx.accounts.mint.try_borrow_data()?;
    let mint = PodStateWithExtensions::<PodMint>::unpack(&mint_borrow)?;
    let decimals = mint.base.decimals;

    // Harvest rewards & update last claim tracking.
    harvest(
        HarvestAccounts {
            config: ctx.accounts.config,
            vault_holder_rewards: ctx.accounts.vault_holder_rewards,
            authority: ctx.accounts.stake_authority,
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

    // Validate the amount.
    require!(
        amount <= delegation.staked_amount,
        StakeError::InsufficientStakeAmount
    );
    let max_deactivation_amount = (delegation.staked_amount as u128)
        .checked_mul(config.max_deactivation_basis_points as u128)
        .and_then(|p| p.checked_div(MAX_BASIS_POINTS))
        .and_then(|amount| u64::try_from(amount).ok())
        .ok_or(ProgramError::ArithmeticOverflow)?;
    require!(
        amount <= max_deactivation_amount,
        StakeError::MaximumDeactivationAmountExceeded,
        "amount requested ({}), maximum allowed ({})",
        amount,
        max_deactivation_amount
    );

    // Update staked amount & unstake cooldown.
    let staked_amount = delegation
        .staked_amount
        .checked_sub(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    delegation.staked_amount = staked_amount;
    delegation.unstake_cooldown = now.saturating_add(config.cooldown_time_seconds);

    sync_effective(config, delegation, (lamports, lamports_min))?;

    drop(mint_borrow);
    drop(vault_borrow);
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
    )?;

    Ok(())
}
