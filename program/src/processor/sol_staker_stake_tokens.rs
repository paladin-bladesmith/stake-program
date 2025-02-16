use solana_program::{entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey};
use spl_token_2022::{
    extension::PodStateWithExtensions,
    pod::{PodAccount, PodMint},
};

use crate::{
    error::StakeError,
    instruction::accounts::{Context, SolStakerStakeTokensAccounts},
    processor::{harvest, sync_effective, unpack_initialized_mut, HarvestAccounts},
    require,
    state::{
        find_sol_staker_stake_pda, find_validator_stake_pda, Config, SolStakerStake, ValidatorStake,
    },
};

/// Stakes tokens with the given config.
///
/// NOTE: This instruction is used by SOL staker stake accounts. The total amount of staked
/// tokens is limited to the 1.3 * current amount of SOL staked by the SOL staker.
///
/// 0. `[w]` Config
/// 1. `[w]` Validator stake
/// 2. `[w]` Sol staker stake
/// 3. `[w]` Sol staker stake authority
/// 4. `[w]` Source token account
/// 5. `[s]` Source token account authority (owner or delegate)
/// 6. `[ ]` Mint
/// 7. `[w]` Vault token account
/// 8. `[w]` Vault holder rewards
/// 9. `[ ]` Token program
/// 10. Extra accounts required for the transfer hook
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
    let mut config = ctx.accounts.config.data.borrow_mut();
    let config = unpack_initialized_mut::<Config>(&mut config)?;

    // validator stake
    // - owner must be the stake program
    // - must be initialized
    // - must have the correct derivation
    require!(
        ctx.accounts.validator_stake.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "validator stake"
    );
    let mut validator_stake = ctx.accounts.validator_stake.try_borrow_mut_data()?;
    let validator_stake = unpack_initialized_mut::<ValidatorStake>(&mut validator_stake)?;
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
        HarvestAccounts {
            config: ctx.accounts.config,
            vault_holder_rewards: ctx.accounts.vault_holder_rewards,
            authority: ctx.accounts.sol_staker_stake_authority,
        },
        config,
        &mut sol_staker_stake.delegation,
        None,
    )?;

    // vault
    // - must be the token account on the stake config account
    require!(
        ctx.accounts.vault.key == &config.vault,
        StakeError::IncorrectVaultAccount,
    );
    let vault_data = ctx.accounts.vault.try_borrow_data()?;
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
        .staked_amount
        .checked_add(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    // Update states.
    sol_staker_stake.delegation.staked_amount = staker_active;
    sync_effective(
        config,
        &mut sol_staker_stake.delegation,
        (sol_staker_stake.lamports_amount, 0),
    )?;
    validator_stake.stakers_total_staked_pal = validator_stake
        .stakers_total_staked_pal
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
        ctx.accounts.source_token_account_authority.clone(),
        ctx.remaining_accounts,
        amount,
        decimals,
        &[],
    )?;

    Ok(())
}
