use solana_program::{
    entrypoint::ProgramResult, program::invoke, program_error::ProgramError, program_pack::Pack,
    pubkey::Pubkey,
};
use spl_token::{
    instruction::transfer_checked,
    state::{Account as TokenAccount, Mint},
};

use crate::{
    error::StakeError,
    instruction::accounts::{Context, SolStakerStakeTokensAccounts},
    processor::{harvest, sync_effective, unpack_initialized_mut, HarvestAccounts},
    require,
    state::{find_sol_staker_stake_pda, find_vault_pda, Config, SolStakerStake},
};

/// Stakes tokens with the given config.
///
/// NOTE: This instruction is used by SOL staker stake accounts. The total amount of staked
/// tokens is limited to the 1.3 * current amount of SOL staked by the SOL staker.
///
/// 0. `[w]` Config
/// 1. `[w]` Sol staker stake
/// 2. `[w]` Sol staker stake authority
/// 3. `[w]` Source token account
/// 4. `[s]` Source token account authority (owner or delegate)
/// 5. `[ ]` Mint
/// 6. `[w]` Vault token account
/// 7. `[w]` Vault holder rewards
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
    let vault_authority = find_vault_pda(ctx.accounts.config.key, program_id).0;
    harvest(
        HarvestAccounts {
            config: ctx.accounts.config,
            vault_holder_rewards: ctx.accounts.vault_holder_rewards,
            authority: ctx.accounts.sol_staker_stake_authority,
        },
        config,
        &vault_authority,
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
    let vault = TokenAccount::unpack(&vault_data)?;

    // mint
    // - must match the stake vault mint
    require!(
        &vault.mint == ctx.accounts.mint.key,
        StakeError::InvalidMint,
        "mint"
    );
    let mint_data = ctx.accounts.mint.try_borrow_data()?;
    let mint = Mint::unpack(&mint_data)?;
    let decimals = mint.decimals;

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

    // Transfer the tokens to the vault (stakes them).
    drop(mint_data);
    drop(vault_data);
    invoke(
        &transfer_checked(
            &spl_token::ID,
            ctx.accounts.source_token_account.key,
            ctx.accounts.mint.key,
            ctx.accounts.vault.key,
            ctx.accounts.source_token_account_authority.key,
            &[],
            amount,
            decimals,
        )?,
        &[
            ctx.accounts.source_token_account.clone(),
            ctx.accounts.mint.clone(),
            ctx.accounts.vault.clone(),
            ctx.accounts.source_token_account_authority.clone(),
        ],
    )?;

    // TODO: Deposit tokens into holder rewards
    Ok(())
}
