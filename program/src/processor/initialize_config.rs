use solana_program::{entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey};
use spl_discriminator::SplDiscriminate;
use spl_pod::optional_keys::OptionalNonZeroPubkey;
use spl_token_2022::{
    extension::{transfer_hook::TransferHook, BaseStateWithExtensions, PodStateWithExtensions},
    pod::{PodAccount, PodMint},
};

use crate::{
    error::StakeError,
    instruction::accounts::{Context, InitializeConfigAccounts},
    processor::REWARDS_PROGRAM_ID,
    require,
    state::Config,
};

/// Creates Stake config account which controls staking parameters.
///
/// ### Accounts:
///
///   0. `[w]` config
///   1. `[]` slash_authority
///   2. `[]` config_authority
///   3. `[]` mint
///   4. `[]` vault
pub fn process_initialize_config(
    program_id: &Pubkey,
    ctx: Context<InitializeConfigAccounts>,
    cooldown_time_seconds: u64,
    max_deactivation_basis_points: u16,
) -> ProgramResult {
    // Accounts validation.

    // 1. mint
    // - owner must the spl token 2022
    // - must be initialized
    // - have rewards transfer hook

    require!(
        ctx.accounts.mint.owner == &spl_token_2022::ID,
        ProgramError::InvalidAccountOwner,
        "mint"
    );

    let mint_data = ctx.accounts.mint.try_borrow_data()?;
    // unpack checks if the mint is initialized
    let mint = PodStateWithExtensions::<PodMint>::unpack(&mint_data)?;

    // ensure the mint is configured with the expected `TransferHook` extension
    let transfer_hook = mint.get_extension::<TransferHook>()?;
    let hook_program_id: Option<Pubkey> = transfer_hook.program_id.into();

    require!(
        hook_program_id == Some(REWARDS_PROGRAM_ID),
        StakeError::InvalidTransferHookProgramId,
        "expected {}, found {:?}",
        program_id,
        hook_program_id
    );

    // 2. vault (token account)
    // - must be initialized
    // - have the vault signer (PDA) as owner
    // - have the correct mint
    // - amount equal to 0

    let vault_data = ctx.accounts.vault.try_borrow_data()?;
    // unpack checks if the token is initialized
    let vault = PodStateWithExtensions::<PodAccount>::unpack(&vault_data)?;

    let (vault_signer, bump) = Pubkey::find_program_address(
        &["token-owner".as_bytes(), ctx.accounts.config.key.as_ref()],
        program_id,
    );

    require!(
        vault.base.owner == vault_signer,
        StakeError::InvalidTokenOwner,
        "vault"
    );

    require!(
        &vault.base.mint == ctx.accounts.mint.key,
        StakeError::InvalidMint,
        "vault"
    );

    let amount: u64 = vault.base.amount.into();
    require!(amount == 0, StakeError::AmountGreaterThanZero, "vault");

    // 3. config
    // - owner must be stake program
    // - have the correct length
    // - be uninitialized

    let mut data = ctx.accounts.config.try_borrow_mut_data()?;

    require!(
        ctx.accounts.config.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "config"
    );

    require!(
        data.len() == Config::LEN,
        StakeError::InvalidAccountDataLength,
        "config"
    );

    let config = bytemuck::from_bytes_mut::<Config>(&mut data);

    require!(
        !config.is_initialized(),
        ProgramError::AccountAlreadyInitialized,
        "config"
    );

    // Initialize the stake config account.

    *config = Config {
        discriminator: Config::SPL_DISCRIMINATOR.into(),
        authority: OptionalNonZeroPubkey(*ctx.accounts.config_authority.key),
        slash_authority: OptionalNonZeroPubkey(*ctx.accounts.slash_authority.key),
        vault: *ctx.accounts.vault.key,
        cooldown_time_seconds: cooldown_time_seconds as i64,
        max_deactivation_basis_points,
        ..Config::default()
    };
    // store the vault signer bump
    config.set_signer_bump(bump);

    Ok(())
}
