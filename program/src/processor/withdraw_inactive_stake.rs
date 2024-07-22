use solana_program::{entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey};
use spl_token_2022::{
    extension::PodStateWithExtensions,
    pod::{PodAccount, PodMint},
};

use crate::{
    error::StakeError,
    instruction::accounts::{Context, WithdrawInactiveStakeAccounts},
    require,
    state::{create_vault_pda, find_stake_pda, get_vault_pda_signer_seeds, Config, Stake},
};

/// Withdraw inactive staked tokens from the vault
///
/// After a deactivation has gone through the cooldown period and been
/// "inactivated", the authority may move the tokens out of the vault.
///
/// 0. `[w]` Config account
/// 1. `[w]` Stake account
/// 2. `[w]` Vault token account
/// 3. `[w]` Destination token account
/// 4. `[s]` Stake authority
/// 5. `[]` Vault authority, PDA with seeds `['token-owner', stake_config]`
/// 6. `[]` SPL Token program
/// 7.. Extra required accounts for transfer hook
///
/// Instruction data: amount of tokens to move
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

    let mut config_data = ctx.accounts.config.try_borrow_mut_data()?;
    let config = bytemuck::try_from_bytes_mut::<Config>(&mut config_data)
        .map_err(|_error| ProgramError::InvalidAccountData)?;

    require!(
        config.is_initialized(),
        ProgramError::UninitializedAccount,
        "config",
    );

    // stake
    // - owner must be the stake program
    // - must be initialized
    // - derivation must match (validates the config account)

    require!(
        ctx.accounts.stake.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "stake"
    );

    let mut stake_data = ctx.accounts.stake.try_borrow_mut_data()?;
    let stake = bytemuck::try_from_bytes_mut::<Stake>(&mut stake_data)
        .map_err(|_error| ProgramError::InvalidAccountData)?;

    require!(
        stake.is_initialized(),
        ProgramError::UninitializedAccount,
        "stake",
    );

    let (derivation, _) =
        find_stake_pda(&stake.validator_vote, ctx.accounts.config.key, program_id);

    require!(
        ctx.accounts.stake.key == &derivation,
        ProgramError::InvalidSeeds,
        "stake",
    );

    // stake authority
    // - must be a signer
    // - must match the authority on the stake account

    require!(
        ctx.accounts.stake_authority.is_signer,
        ProgramError::MissingRequiredSignature,
        "stake authority",
    );

    require!(
        ctx.accounts.stake_authority.key == &stake.authority,
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

    require!(
        amount <= stake.inactive_amount,
        StakeError::NotEnoughInactivatedTokens,
        "amount"
    );

    stake.inactive_amount = stake
        .inactive_amount
        .checked_sub(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    config.token_amount_delegated = config
        .token_amount_delegated
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
