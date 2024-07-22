use solana_program::{
    entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey, vote::state::VoteState,
};
use spl_token_2022::{
    extension::PodStateWithExtensions,
    pod::{PodAccount, PodMint},
};

use crate::{
    error::StakeError,
    instruction::accounts::{Context, StakeTokensAccounts},
    require,
    state::{find_stake_pda, Config, Stake},
};

/// Stakes tokens with the given config.
///
/// Limited to the current amount of SOL staked to the validator.
///
/// NOTE: Anybody can stake tokens to a validator, but this does not work
/// like native staking, because the validator can take control of staked
/// tokens by deactivating and withdrawing.
///
/// 0. `[w]` Stake config account
/// 1. `[w]` Validator stake account
///          (PDA seeds: ['stake', validator, config_account])
/// 2. `[w]` Token Account
/// 3. `[s]` Owner or delegate of the token account
/// 4. `[ ]` Validator vote account
/// 5. `[ ]` Stake Token Mint
/// 6. `[w]` Stake Token Vault, to hold all staked tokens
///          (must be the token account on the stake config account)
/// 7. `[ ]` Token program
/// 8.. Extra accounts required for the transfer hook
///
/// Instruction data: amount of tokens to stake, as a little-endian u64
pub fn process_stake_tokens<'a>(
    program_id: &Pubkey,
    ctx: Context<'a, StakeTokensAccounts<'a>>,
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

    // validator vote
    // - owner must be the vote program
    // - must be initialized

    require!(
        ctx.accounts.validator_vote.owner == &solana_program::vote::program::ID,
        ProgramError::InvalidAccountOwner,
        "validator vote"
    );

    let validator_vote_data = ctx.accounts.validator_vote.try_borrow_data()?;

    require!(
        VoteState::is_correct_size_and_initialized(&validator_vote_data),
        ProgramError::InvalidAccountData,
        "validator vote"
    );

    // stake
    // - owner must be the stake program
    // - must have the correct derivation
    // - must be initialized

    require!(
        ctx.accounts.stake.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "stake"
    );

    let (derivation, _) = find_stake_pda(
        ctx.accounts.validator_vote.key,
        ctx.accounts.config.key,
        program_id,
    );

    require!(
        ctx.accounts.stake.key == &derivation,
        ProgramError::InvalidSeeds,
        "stake",
    );

    let mut stake_data = ctx.accounts.stake.try_borrow_mut_data()?;
    let stake = bytemuck::try_from_bytes_mut::<Stake>(&mut stake_data)
        .map_err(|_error| ProgramError::InvalidAccountData)?;

    require!(
        stake.is_initialized(),
        ProgramError::UninitializedAccount,
        "stake",
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
    //
    // Note: validate the amount against the total SOL staked on the validator.

    require!(amount > 0, StakeError::InvalidAmount);

    config.token_amount_delegated = config
        .token_amount_delegated
        .checked_add(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    stake.amount = stake
        .amount
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
