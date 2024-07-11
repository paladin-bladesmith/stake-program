use solana_program::{
    entrypoint::ProgramResult, program::invoke_signed, program_error::ProgramError, pubkey::Pubkey,
};
use spl_token_2022::{
    extension::PodStateWithExtensions,
    instruction::burn_checked,
    pod::{PodAccount, PodMint},
};

use crate::{
    err,
    error::StakeError,
    instruction::accounts::{Context, SlashAccounts},
    require,
    state::{find_stake_pda, find_vault_pda, get_stake_pda_signer_seeds, Config, Stake},
};

/// Slashes a stake account for the given amount
///
/// Burns the given amount of tokens from the vault account, and reduces the
/// amount in the stake account.
///
/// 0. `[w]` Config account
/// 1. `[w]` Stake account
/// 2. `[s]` Slash authority
/// 3. `[w]` Vault token account
/// 4. `[]` Stake Token Mint
/// 5. `[]` Vault authority, PDA with seeds `['token-owner', stake_config]`
/// 6. `[]` SPL Token program
///
/// Instruction data: amount of tokens to slash
pub fn process_slash(
    program_id: &Pubkey,
    ctx: Context<SlashAccounts>,
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

    // slash authority
    // - must be a signer
    // - must match the slash authority on the config account
    //
    // When there is no slash authority set, the stake account cannot be slashed and
    // an error is returned.

    let slash_authority = config.slash_authority.into();

    if let Some(slash_authority) = slash_authority {
        require!(
            ctx.accounts.slash_authority.key == &slash_authority,
            StakeError::InvalidAuthority,
            "slash authority",
        );
    } else {
        return err!(StakeError::AuthorityNotSet, "slash authority");
    }

    require!(
        ctx.accounts.slash_authority.is_signer,
        ProgramError::MissingRequiredSignature,
        "stake authority",
    );

    // vault authority
    // - derivation must match

    let (vault_signer, bump) = find_vault_pda(ctx.accounts.config.key, program_id);

    require!(
        ctx.accounts.vault_authority.key == &vault_signer,
        StakeError::InvalidAuthority,
        "vault authority",
    );

    let signer_bump = &[bump];
    let signer_seeds =
        get_stake_pda_signer_seeds(&stake.validator, ctx.accounts.config.key, signer_bump);

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

    // Update the stake amount on both config and stake accounts.

    require!(
        config.token_amount_delegated >= amount,
        StakeError::InsufficientStakeAmount,
        "config"
    );

    config.token_amount_delegated = config
        .token_amount_delegated
        .checked_sub(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    require!(
        stake.amount >= amount,
        StakeError::InsufficientStakeAmount,
        "stake"
    );

    stake.amount = stake
        .amount
        .checked_sub(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    // Burn the tokens from the vault account.

    let mint_data = ctx.accounts.mint.try_borrow_data()?;
    let mint = PodStateWithExtensions::<PodMint>::unpack(&mint_data)?;
    let decimals = mint.base.decimals;

    drop(mint_data);
    drop(vault_data);

    let transfer_ix = burn_checked(
        ctx.accounts.token_program.key,
        ctx.accounts.vault.key,
        ctx.accounts.mint.key,
        ctx.accounts.vault_authority.key,
        &[],
        amount,
        decimals,
    )?;

    invoke_signed(
        &transfer_ix,
        &[
            ctx.accounts.token_program.clone(),
            ctx.accounts.vault.clone(),
            ctx.accounts.mint.clone(),
            ctx.accounts.vault_authority.clone(),
        ],
        &[&signer_seeds],
    )
}
