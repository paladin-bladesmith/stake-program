use solana_program::{entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey};
use spl_token_2022::{
    extension::PodStateWithExtensions,
    pod::{PodAccount, PodMint},
};

use crate::{
    error::StakeError,
    instruction::accounts::{Context, SolStakerStakeTokensAccounts},
    processor::{harvest, unpack_initialized_mut, HarvestAccounts},
    require,
    state::{
        calculate_maximum_stake_for_lamports_amount, find_sol_staker_stake_pda, Config,
        SolStakerStake,
    },
};

/// Stakes tokens with the given config.
///
/// NOTE: This instruction is used by SOL staker stake accounts. The total amount of staked
/// tokens is limited to the 1.3 * current amount of SOL staked by the SOL staker.
///
/// 0. `[w]` Stake config account
/// 1. `[w]` SOL staker stake account
///          (PDA seeds: ['stake::state::sol_staker_stake', validator, config_account])
/// 2. `[w]` SOL staker stake authority account
/// 3. `[w]` Token Account
/// 4. `[s]` Owner or delegate of the token account
/// 5. `[ ]` Stake Token Mint
/// 6. `[w]` Stake Token Vault, to hold all staked tokens
///          (must be the token account on the stake config account)
/// 7. `[w]` Stake Token Vault holder rewards
/// 8. `[ ]` Token program
/// 9. Extra accounts required for the transfer hook
///
/// Instruction data: amount of tokens to stake, as a little-endian `u64`.
pub fn process_sol_staker_stake_tokens<'a>(
    program_id: &Pubkey,
    ctx: Context<'a, SolStakerStakeTokensAccounts<'a>>,
    amount: u64,
) -> ProgramResult {
    // Account validation.

    // sol staker stake
    // - owner must be the stake program
    // - must be initialized
    // - must have the correct derivation (validates the config account)
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
        "sol staker stake",
    );

    // Harvest rewards & update last claim tracking.
    harvest(
        program_id,
        HarvestAccounts {
            config: ctx.accounts.config,
            vault_holder_rewards: ctx.accounts.vault_holder_rewards,
            authority: ctx.accounts.sol_staker_stake_authority,
        },
        &mut sol_staker_stake.delegation,
        None,
    )?;

    // config
    // - owner must be the stake program
    // - must be initialized
    require!(
        ctx.accounts.config.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "config"
    );
    let mut config_data = ctx.accounts.config.try_borrow_mut_data()?;
    let config = unpack_initialized_mut::<Config>(&mut config_data)?;

    // vault
    // - must be the token account on the stake config account

    require!(
        ctx.accounts.vault.key == &config.vault,
        StakeError::IncorrectVaultAccount,
    );

    let vault_data = ctx.accounts.vault.try_borrow_data()?;
    // unpack to validate the mint
    let vault = PodStateWithExtensions::<PodAccount>::unpack(&vault_data)?;

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

    // Compute staker total & effective stakes.
    require!(amount > 0, StakeError::InvalidAmount);
    let staker_active = sol_staker_stake
        .delegation
        .active_amount
        .checked_add(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    let staker_limit =
        calculate_maximum_stake_for_lamports_amount(sol_staker_stake.lamports_amount)?;
    let staker_effective = std::cmp::min(staker_active, staker_limit);
    let effective_delta = staker_effective
        .checked_sub(sol_staker_stake.delegation.effective_amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    // Update states.
    sol_staker_stake.delegation.active_amount = staker_active;
    sol_staker_stake.delegation.effective_amount = staker_effective;
    config.token_amount_effective = config
        .token_amount_effective
        .checked_add(effective_delta)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    // Transfer the tokens to the vault (stakes them).
    drop(mint_data);
    drop(vault_data);
    spl_token_2022::onchain::invoke_transfer_checked(
        &spl_token_2022::ID,
        ctx.accounts.source_token_account.clone(),
        ctx.accounts.mint.clone(),
        ctx.accounts.vault.clone(),
        ctx.accounts.token_account_authority.clone(),
        ctx.remaining_accounts,
        amount,
        decimals,
        &[],
    )
}
