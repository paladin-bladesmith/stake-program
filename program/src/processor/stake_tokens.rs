use arrayref::array_ref;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke,
    program_error::ProgramError, pubkey::Pubkey, vote::state::VoteState,
};
use spl_token_2022::{
    extension::PodStateWithExtensions,
    instruction::transfer_checked,
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
///     * PDA seeds: ['stake', validator, config_account]
/// 2. `[w]` Token Account
/// 3. `[s]` Owner or delegate of the token account
/// 4. `[]` Validator vote account
/// 3. `[]` Stake Token Mint
/// 4. `[]` Stake Token Vault, to hold all staked tokens.
///   Must be the token account on the stake config account
/// 5. `[]` Token program
/// 6.. Extra accounts required for the transfer hook
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

    // validator_vote
    // - owner must be the vote program
    // - must be initialized

    require!(
        ctx.accounts.validator_vote.owner == &solana_program::vote::program::ID,
        ProgramError::InvalidAccountOwner,
        "validator_vote"
    );

    let validator_vote_data = ctx.accounts.validator_vote.try_borrow_data()?;

    require!(
        VoteState::is_correct_size_and_initialized(&validator_vote_data),
        ProgramError::InvalidAccountData,
        "validator_vote"
    );

    let validator = Pubkey::from(*array_ref!(validator_vote_data, 4, 32));
    drop(validator_vote_data);

    // stake
    // - owner must be the stake program
    // - must have the correct derivation
    // - must be initialized

    require!(
        ctx.accounts.stake.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "stake"
    );

    let (derivation, _) = find_stake_pda(&validator, ctx.accounts.config.key, program_id);

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
    drop(mint_data);

    // Update the config and stake account data.
    //
    // TODO: validate the amount against the total SOL staked on the validator.

    config.token_amount_delegated = config
        .token_amount_delegated
        .checked_add(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    drop(config_data);

    stake.amount = stake
        .amount
        .checked_add(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    drop(stake_data);

    // Transfer the tokens to the vault (stakes them).

    let transfer_ix = transfer_checked(
        ctx.accounts.token_program.key,
        ctx.accounts.source_token_account.key,
        ctx.accounts.mint.key,
        ctx.accounts.vault.key,
        ctx.accounts.token_account_authority.key,
        &[],
        amount,
        decimals,
    )?;

    let mut account_infos: Vec<AccountInfo> = Vec::with_capacity(5 + ctx.remaining_accounts.len());
    account_infos.push(ctx.accounts.token_program.clone());
    account_infos.push(ctx.accounts.source_token_account.clone());
    account_infos.push(ctx.accounts.mint.clone());
    account_infos.push(ctx.accounts.vault.clone());
    account_infos.push(ctx.accounts.token_account_authority.clone());
    account_infos.extend_from_slice(ctx.remaining_accounts);

    invoke(&transfer_ix, &account_infos)
}
