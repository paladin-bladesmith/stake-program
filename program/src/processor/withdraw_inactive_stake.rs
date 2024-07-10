use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke,
    program_error::ProgramError, pubkey::Pubkey,
};
use spl_token_2022::{
    extension::PodStateWithExtensions,
    instruction::transfer_checked,
    pod::{PodAccount, PodMint},
};

use crate::{
    error::StakeError,
    instruction::accounts::{Context, WithdrawInactiveStakeAccounts},
    require,
    state::{find_stake_pda, find_vault_pda, Config, Stake},
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
pub fn process_withdraw_inactive_stake(
    program_id: &Pubkey,
    ctx: Context<WithdrawInactiveStakeAccounts>,
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

    let (derivation, _) = find_stake_pda(&stake.validator, ctx.accounts.config.key, program_id);

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

    let (vault_signer, signer_bump) = find_vault_pda(ctx.accounts.config.key, program_id);

    require!(
        ctx.accounts.vault_authority.key == &vault_signer,
        StakeError::InvalidAuthority,
        "vault authority",
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

    require!(
        amount <= stake.inactive_amount,
        StakeError::ActiveDeactivationCooldown,
        "amount"
    );

    stake.inactive_amount = stake
        .inactive_amount
        .checked_sub(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    stake.amount = stake
        .amount
        .checked_sub(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    drop(stake_data);

    config.token_amount_delegated = config
        .token_amount_delegated
        .checked_sub(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    drop(config_data);

    // Transfer the tokens to the vault (stakes them).

    let transfer_ix = transfer_checked(
        ctx.accounts.token_program.key,
        ctx.accounts.vault.key,
        ctx.accounts.mint.key,
        ctx.accounts.destination_token_account.key,
        ctx.accounts.vault_authority.key,
        &[],
        amount,
        decimals,
    )?;

    let mut account_infos: Vec<AccountInfo> = Vec::with_capacity(5 + ctx.remaining_accounts.len());
    account_infos.push(ctx.accounts.token_program.clone());
    account_infos.push(ctx.accounts.vault.clone());
    account_infos.push(ctx.accounts.mint.clone());
    account_infos.push(ctx.accounts.destination_token_account.clone());
    account_infos.push(ctx.accounts.vault_authority.clone());
    account_infos.extend_from_slice(ctx.remaining_accounts);

    invoke(&transfer_ix, &account_infos)
}
