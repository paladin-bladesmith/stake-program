use std::cmp::min;

use solana_program::{
    entrypoint::ProgramResult, msg, program::invoke_signed, program_error::ProgramError,
    pubkey::Pubkey,
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
    state::{
        create_vault_pda, find_validator_stake_pda, get_vault_pda_signer_seeds, Config,
        SolStakerStake,
    },
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
    let stake = bytemuck::try_from_bytes_mut::<SolStakerStake>(&mut stake_data)
        .map_err(|_error| ProgramError::InvalidAccountData)?;

    require!(
        stake.is_initialized(),
        ProgramError::UninitializedAccount,
        "stake",
    );

    let (derivation, _) =
        find_validator_stake_pda(&stake.validator_vote, ctx.accounts.config.key, program_id);

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

    if let Some(slash_authority) = Option::<Pubkey>::from(config.slash_authority) {
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

    let signer_bump = [config.vault_authority_bump];
    let derivation = create_vault_pda(ctx.accounts.config.key, &signer_bump, program_id)?;

    require!(
        ctx.accounts.vault_authority.key == &derivation,
        StakeError::InvalidAuthority,
        "vault authority",
    );

    let signer_seeds = get_vault_pda_signer_seeds(ctx.accounts.config.key, &signer_bump);

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

    // Update the stake amount on both stake and config accounts:
    //
    //   1. the amount slashed is taken from the stake amount (this includes
    //      the amount that is currently staked + deactivating);
    //
    //   2. if not enough, the remaining is ignored and the stake account is
    //      left with 0 amount;
    //
    //   3. if the stake account has deactivating tokens, make sure that the
    //      deactivating amount is at least the same as the stake amount (it might
    //      need to be reduced due to slashing).

    require!(
        amount > 0,
        StakeError::InvalidAmount,
        "amount must be greater than 0"
    );

    // slashes active stake
    let active_stake_to_slash = min(amount, stake.amount);
    stake.amount = stake
        .amount
        .checked_sub(active_stake_to_slash)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    if stake.deactivating_amount > stake.amount {
        // adjust the deactivating amount if it's greater than the "active" stake
        // after slashing
        stake.deactivating_amount = stake.amount;
        // clear the deactivation timestamp if there in no deactivating
        // amount left
        if stake.deactivating_amount == 0 {
            stake.deactivation_timestamp = None;
        }
    }

    // Update the token amount delegated on the config account.
    //
    // The instruction will fail if the amount to slash is greater than the
    // total amount delegated (it should never happen).
    require!(
        config.token_amount_delegated >= active_stake_to_slash,
        StakeError::InvalidSlashAmount,
        "slash amount greater than total amount delegated ({} required, {} delegated)",
        active_stake_to_slash,
        config.token_amount_delegated
    );

    config.token_amount_delegated = config
        .token_amount_delegated
        .checked_sub(active_stake_to_slash)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    if amount > active_stake_to_slash {
        // The amount to slash was greater than the amount available on the
        // stake account.
        msg!(
            "Slash amount greater than available tokens on stake account ({} required, {} available)",
            amount,
            active_stake_to_slash,
        );
    }

    // Burn the tokens from the vault account (if there are tokens to slash).

    if active_stake_to_slash > 0 {
        let mint_data = ctx.accounts.mint.try_borrow_data()?;
        let mint = PodStateWithExtensions::<PodMint>::unpack(&mint_data)?;
        let decimals = mint.base.decimals;

        drop(mint_data);
        drop(vault_data);

        let burn_ix = burn_checked(
            ctx.accounts.token_program.key,
            ctx.accounts.vault.key,
            ctx.accounts.mint.key,
            ctx.accounts.vault_authority.key,
            &[],
            active_stake_to_slash,
            decimals,
        )?;

        invoke_signed(
            &burn_ix,
            &[
                ctx.accounts.token_program.clone(),
                ctx.accounts.vault.clone(),
                ctx.accounts.mint.clone(),
                ctx.accounts.vault_authority.clone(),
            ],
            &[&signer_seeds],
        )?;
    }

    Ok(())
}
