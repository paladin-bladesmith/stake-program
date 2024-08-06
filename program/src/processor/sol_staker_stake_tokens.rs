use solana_program::{entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey};
use spl_token_2022::{
    extension::PodStateWithExtensions,
    pod::{PodAccount, PodMint},
};

use crate::{
    error::StakeError,
    instruction::accounts::{Context, SolStakerStakeTokensAccounts},
    processor::unpack_initialized_mut,
    require,
    state::{
        calculate_maximum_stake_for_lamports_amount, find_sol_staker_stake_pda,
        find_validator_stake_pda, Config, SolStakerStake, ValidatorStake,
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
/// 2. `[w]` Validator stake account
///          (PDA seeds: ['stake::state::validator_stake', validator, config_account])
/// 3. `[w]` Token Account
/// 4. `[s]` Owner or delegate of the token account
/// 5. `[ ]` Stake Token Mint
/// 6. `[w]` Stake Token Vault, to hold all staked tokens
///          (must be the token account on the stake config account)
/// 7. `[ ]` Token program
/// 8.. Extra accounts required for the transfer hook
///
/// Instruction data: amount of tokens to stake, as a little-endian `u64`.
pub fn process_sol_staker_stake_tokens<'a>(
    program_id: &Pubkey,
    ctx: Context<'a, SolStakerStakeTokensAccounts<'a>>,
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

    let mut config_data = ctx.accounts.config.try_borrow_mut_data()?;
    let config = unpack_initialized_mut::<Config>(&mut config_data)?;

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

    // validator stake
    // - owner must be the stake program
    // - must be initialized
    // - must have the correct derivation (validator vote must match the validator vote in
    //   the sol staker stake account)

    require!(
        ctx.accounts.validator_stake.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "validator stake"
    );

    let mut validator_stake_data = ctx.accounts.validator_stake.try_borrow_mut_data()?;
    let validator_stake = unpack_initialized_mut::<ValidatorStake>(&mut validator_stake_data)?;

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

    // Update the config and stake account data.

    require!(amount > 0, StakeError::InvalidAmount);

    let updated_staked_amount = sol_staker_stake
        .delegation
        .amount
        .checked_add(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    // maximum allowed stake based on the SOL amount
    let limit = calculate_maximum_stake_for_lamports_amount(sol_staker_stake.lamports_amount)?;

    require!(
        updated_staked_amount as u128 <= limit,
        StakeError::TotalStakeAmountExceedsSolLimit,
        "current staked amount ({}) + new amount ({}) exceeds limit ({})",
        sol_staker_stake.delegation.amount,
        amount,
        limit
    );
    // update the degation amount of the SOL staker stake account
    sol_staker_stake.delegation.amount = updated_staked_amount;
    // update the total tokens amount of the validator stake account
    validator_stake.total_staked_token_amount = validator_stake
        .total_staked_token_amount
        .checked_add(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    // update the total amount delegated on the config account
    config.token_amount_delegated = config
        .token_amount_delegated
        .checked_add(amount)
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
