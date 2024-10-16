use solana_program::{entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey};
use spl_token_2022::{
    extension::PodStateWithExtensions,
    pod::{PodAccount, PodMint},
};

use crate::{
    error::StakeError,
    instruction::accounts::{Context, ValidatorStakeTokensAccounts},
    processor::{harvest, unpack_initialized_mut, HarvestAccounts},
    require,
    state::{
        calculate_maximum_stake_for_lamports_amount, find_validator_stake_pda, Config,
        ValidatorStake,
    },
};

/// Stakes tokens with the given config.
///
/// NOTE: This instruction is used by validator stake accounts. The total amount of effective
/// staked tokens is limited to the 1.3 * current amount of SOL staked to the validator.
///
/// 0. `[w]` Stake config account
/// 1. `[w]` Validator stake account
///          (PDA seeds: ['stake::state::validator_stake', validator, config_account])
/// 2. `[w]` Validator stake authority account
/// 3. `[w]` Token Account
/// 4. `[s]` Owner or delegate of the token account
/// 5. `[ ]` Stake Token Mint
/// 6. `[w]` Stake Token Vault, to hold all staked tokens
///          (must be the token account on the stake config account)
/// 7. `[ ]` Token program
/// 8. Extra accounts required for the transfer hook
///
/// Instruction data: amount of tokens to stake, as a little-endian `u64`.
pub fn process_validator_stake_tokens<'a>(
    program_id: &Pubkey,
    ctx: Context<'a, ValidatorStakeTokensAccounts<'a>>,
    amount: u64,
) -> ProgramResult {
    // Account validation.

    // validator stake
    // - owner must be the stake program
    // - must be initialized
    // - must have the correct derivation
    require!(
        ctx.accounts.validator_stake.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "validator stake"
    );
    let mut stake_data = ctx.accounts.validator_stake.try_borrow_mut_data()?;
    let stake = unpack_initialized_mut::<ValidatorStake>(&mut stake_data)?;
    let (derivation, _) = find_validator_stake_pda(
        &stake.delegation.validator_vote,
        ctx.accounts.config.key,
        program_id,
    );
    require!(
        ctx.accounts.validator_stake.key == &derivation,
        ProgramError::InvalidSeeds,
        "validator stake",
    );

    // Harvest rewards & update last claim tracking.
    harvest(
        HarvestAccounts {
            config: ctx.accounts.config,
            holder_rewards: ctx.accounts.vault_holder_rewards,
            recipient: ctx.accounts.validator_stake_authority,
        },
        &mut stake.delegation,
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
    let config = bytemuck::try_from_bytes_mut::<Config>(&mut config_data)
        .map_err(|_error| ProgramError::InvalidAccountData)?;
    require!(
        config.is_initialized(),
        ProgramError::UninitializedAccount,
        "config",
    );

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

    // Compute the new total & effective stakes.
    require!(amount > 0, StakeError::InvalidAmount);
    let validator_active = stake
        .delegation
        .active_amount
        .checked_add(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    let validator_limit =
        calculate_maximum_stake_for_lamports_amount(stake.total_staked_lamports_amount)?;
    let validator_effective = std::cmp::min(validator_active, validator_limit);
    let effective_delta = validator_effective
        .checked_sub(stake.delegation.effective_amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    // Update states.
    stake.delegation.active_amount = validator_active;
    stake.delegation.effective_amount = validator_effective;
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
