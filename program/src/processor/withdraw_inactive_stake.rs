use solana_program::{entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey};
use spl_token_2022::{
    extension::PodStateWithExtensions,
    pod::{PodAccount, PodMint},
};

use crate::{
    error::StakeError,
    instruction::accounts::{Context, WithdrawInactiveStakeAccounts},
    processor::{harvest, unpack_delegation_mut_checked, unpack_initialized_mut, HarvestAccounts},
    require,
    state::{create_vault_pda, get_vault_pda_signer_seeds, Config},
};

/// Withdraw inactive staked tokens from the vault.
///
/// After a deactivation has gone through the cooldown period and been
/// "inactivated", the authority may move the tokens out of the vault.
///
/// 0. `[w]` Config
/// 1. `[w]` Validator or Sol staker stake
/// 2. `[w]` Vault token account
/// 3. `[ ]` Mint
/// 4. `[w]` Destination token account
/// 5. `[s]` Stake authority
/// 6. `[ ]` Vault authority
/// 7. `[ ]` Token program
/// 8. Extra required accounts for transfer hook
///
/// Instruction data: amount of tokens to move.
pub fn process_withdraw_inactive_stake<'a>(
    program_id: &Pubkey,
    ctx: Context<'a, WithdrawInactiveStakeAccounts<'a>>,
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
    let mut config = ctx.accounts.config.try_borrow_mut_data()?;
    let config = unpack_initialized_mut::<Config>(&mut config)?;
    assert!(config.is_initialized());

    // stake
    // - owner must be the stake program
    // - must be initialized
    // - derivation must match (validates the config account)
    require!(
        ctx.accounts.stake.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "stake"
    );
    let stake_data = &mut ctx.accounts.stake.try_borrow_mut_data()?;
    let delegation = unpack_delegation_mut_checked(
        stake_data,
        ctx.accounts.stake.key,
        ctx.accounts.config.key,
        program_id,
    )?;

    // stake authority
    // - must be a signer
    // - must match the authority on the stake account
    require!(
        ctx.accounts.stake_authority.is_signer,
        ProgramError::MissingRequiredSignature,
        "stake authority",
    );
    require!(
        ctx.accounts.stake_authority.key == &delegation.authority,
        StakeError::InvalidAuthority,
        "stake authority",
    );

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

    // Harvest any pending rewards before updating amounts.
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

    // Update the config and stake account data.
    require!(
        amount <= delegation.inactive_amount,
        StakeError::NotEnoughInactivatedTokens,
        "amount"
    );
    delegation.inactive_amount = delegation
        .inactive_amount
        .checked_sub(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    // Transfer the tokens from the vault to destination (unstakes them).
    drop(vault_data);
    drop(mint_data);
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
    )
}
