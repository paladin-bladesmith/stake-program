use paladin_rewards_program_client::instructions::DepositBuilder;
use solana_program::program::invoke_signed;
use solana_program::{
    entrypoint::ProgramResult, program::invoke, program_error::ProgramError, program_pack::Pack,
    pubkey::Pubkey,
};
use spl_token::state::Account as TokenAccount;
use spl_token::{instruction::transfer_checked, state::Mint};

use crate::state::get_vault_pda_signer_seeds;
use crate::{
    error::StakeError,
    instruction::accounts::{Context, ValidatorStakeTokensAccounts},
    processor::{harvest, sync_effective, unpack_initialized_mut, HarvestAccounts},
    require,
    state::{find_validator_stake_pda, find_vault_pda, Config, ValidatorStake},
};

/// Stakes tokens with the given config.
///
/// NOTE: This instruction is used by validator stake accounts. The total amount of effective
/// staked tokens is limited to the 1.3 * current amount of SOL staked to the validator.
///
/// 0. `[w]` Config
/// 1. `[w]` Validator stake
/// 2. `[w]` Validator stake authority
/// 3. `[w]` Source token account
/// 4. `[s]` Source token authority (owner or delegate)
/// 5. `[ ]` Mint
/// 6. `[w]` Vault
/// 7. `[w]` Vault holder rewards
/// 8. `[ ]` Token program
/// 9. Extra accounts required for the transfer hook
///
/// Instruction data: amount of tokens to stake, as a little-endian `u64`.
pub fn process_validator_stake_tokens<'a>(
    program_id: &Pubkey,
    ctx: Context<'a, ValidatorStakeTokensAccounts<'a>>,
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
    let (vault_signer, signer_bump) = find_vault_pda(ctx.accounts.config.key, program_id);
    harvest(
        HarvestAccounts {
            config: ctx.accounts.config,
            vault_holder_rewards: ctx.accounts.vault_holder_rewards,
            authority: ctx.accounts.validator_stake_authority,
        },
        config,
        &vault_signer,
        &mut stake.delegation,
        None,
    )?;

    // Verify vault PDA
    let signer_bump = [signer_bump];
    let vault_seeds = get_vault_pda_signer_seeds(ctx.accounts.config.key, &signer_bump);

    // Ensure the provided vault pda address is the correct address
    require!(
        ctx.accounts.vault_pda.key == &vault_signer,
        StakeError::IncorrectVaultPdaAccount,
        "vault pda"
    );

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

    // Compute the new total & effective stakes.
    require!(amount > 0, StakeError::InvalidAmount);
    let validator_active = stake
        .delegation
        .staked_amount
        .checked_add(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    // Update states.
    stake.delegation.staked_amount = validator_active;
    sync_effective(
        config,
        &mut stake.delegation,
        (
            stake.total_staked_lamports_amount,
            stake.total_staked_lamports_amount_min,
        ),
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

    // Deposit tokens into holder rewards
    invoke_signed(
        &DepositBuilder::new()
            .holder_rewards_pool(*ctx.accounts.holder_rewards_pool.key)
            .holder_rewards_pool_token_account(*ctx.accounts.holder_rewards_pool_token_account.key)
            .owner(*ctx.accounts.vault_pda.key)
            .token_account(*ctx.accounts.vault.key)
            .holder_rewards(*ctx.accounts.vault_holder_rewards.key)
            .mint(*ctx.accounts.mint.key)
            .token_program(*ctx.accounts.token_program.key)
            .amount(amount)
            .instruction(),
        &[
            ctx.accounts.holder_rewards_pool.clone(),
            ctx.accounts.holder_rewards_pool_token_account.clone(),
            ctx.accounts.vault_holder_rewards.clone(),
            ctx.accounts.vault.clone(),
            ctx.accounts.mint.clone(),
            ctx.accounts.vault_pda.clone(),
            ctx.accounts.token_program.clone(),
        ],
        &[&vault_seeds],
    )?;

    Ok(())
}
