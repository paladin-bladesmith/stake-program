use solana_program::{entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey};
use spl_pod::{optional_keys::OptionalNonZeroPubkey, primitives::PodU64};
use spl_token_2022::{
    extension::PodStateWithExtensions,
    pod::{PodAccount, PodMint},
};

use crate::{
    error::StakeError,
    instruction::accounts::{Context, InitializeConfigAccounts},
    require,
    state::{AccountType, Config},
};

/// Creates Stake config account which controls staking parameters.
///
/// ### Accounts:
///
///   0. `[w]` config
///   1. `[]` authority
///   2. `[]` slash_authority
///   3. `[]` mint
///   4. `[]` vault_token
pub fn process_initialize_config(
    program_id: &Pubkey,
    ctx: Context<InitializeConfigAccounts>,
    cooldown_time: u64,
    max_deactivation_basis_points: u16,
) -> ProgramResult {
    // Accounts validation.

    require!(
        ctx.accounts.mint.owner == &spl_token_2022::ID,
        ProgramError::InvalidAccountOwner,
        "mint"
    );

    let mint_data = ctx.accounts.mint.try_borrow_data()?;
    // unpack checks if the mint is initialized
    let _mint = PodStateWithExtensions::<PodMint>::unpack(&mint_data)?;

    // TODO: validate the mint (e.g., transfer hook)

    let token_data = ctx.accounts.vault_token.try_borrow_data()?;
    let token = PodStateWithExtensions::<PodAccount>::unpack(&token_data)?;

    let (vault, bump) = Pubkey::find_program_address(
        &["token-owner".as_bytes(), ctx.accounts.config.key.as_ref()],
        program_id,
    );

    require!(
        token.base.owner == vault,
        StakeError::InvalidTokenOwner,
        "vault_token"
    );

    require!(
        &token.base.mint == ctx.accounts.mint.key,
        ProgramError::UninitializedAccount,
        "mint"
    );

    require!(
        <PodU64 as Into<u64>>::into(token.base.amount) == 0,
        StakeError::AmountGreaterThanZero,
        "vault_token"
    );

    let mut data = ctx.accounts.config.try_borrow_mut_data()?;

    require!(
        ctx.accounts.config.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "config"
    );

    require!(
        data.len() == Config::LEN,
        ProgramError::AccountDataTooSmall,
        "config"
    );

    let config = bytemuck::from_bytes_mut::<Config>(&mut data);

    require!(
        config.account_type == AccountType::Uninitialized,
        ProgramError::AccountAlreadyInitialized,
        "config"
    );

    // Initialize the stake config account.

    config.account_type = AccountType::Config;
    config.authority = OptionalNonZeroPubkey(*ctx.accounts.config_authority.key);
    config.slash_authority = OptionalNonZeroPubkey(*ctx.accounts.slash_authority.key);
    config.vault_token = *ctx.accounts.vault_token.key;
    config.vault_bump = bump;
    config.cooldown_time_seconds = cooldown_time.into();
    config.max_deactivation_basis_points = max_deactivation_basis_points.into();

    Ok(())
}
