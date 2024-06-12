use solana_program::{
    entrypoint::ProgramResult, program::invoke, program_error::ProgramError, pubkey::Pubkey,
    rent::Rent, system_instruction, system_program, sysvar::Sysvar,
};
use spl_pod::{optional_keys::OptionalNonZeroPubkey, primitives::PodU64};
use spl_token_2022::{
    extension::PodStateWithExtensions,
    pod::{PodAccount, PodMint},
};

use crate::{
    err,
    error::StakeError,
    instruction::accounts::{Context, InitializeConfigAccounts},
    require,
    state::{AccountType, Config},
};

/// Creates Stake config account which controls staking parameters.
///
/// ### Accounts:
///
///   0. `[w, s]` config
///   1. `[]` authority
///   2. `[]` slash_authority
///   3. `[]` mint
///   4. `[]` vault_token
///   5. `[optional]` payer
///   6. `[optional]` system_program
pub fn process_initialize_config(
    program_id: &Pubkey,
    ctx: Context<InitializeConfigAccounts>,
    cooldown_time: u64,
    max_deactivation_basis_points: u16,
) -> ProgramResult {
    // Accounts validation.

    require!(
        ctx.accounts.config.is_signer,
        ProgramError::MissingRequiredSignature,
        "config"
    );

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

    if ctx.accounts.config.data_is_empty() {
        let payer = {
            require!(
                ctx.accounts.payer.is_some(),
                ProgramError::NotEnoughAccountKeys,
                "payer"
            );

            ctx.accounts.payer.unwrap()
        };

        require!(
            payer.is_signer,
            ProgramError::MissingRequiredSignature,
            "payer"
        );

        let system_program = {
            require!(
                ctx.accounts.system_program.is_some(),
                ProgramError::NotEnoughAccountKeys,
                "system_program"
            );

            ctx.accounts.system_program.unwrap()
        };

        require!(
            system_program.key == &system_program::ID,
            ProgramError::IncorrectProgramId,
            "system_program"
        );

        invoke(
            &system_instruction::create_account(
                payer.key,
                ctx.accounts.config.key,
                Rent::get()?.minimum_balance(Config::LEN),
                Config::LEN as u64,
                program_id,
            ),
            &[payer.clone(), ctx.accounts.config.clone()],
        )?;
    } else {
        return err!(ProgramError::AccountAlreadyInitialized);
    }

    // Initialize the stake config account.

    let mut data = ctx.accounts.config.try_borrow_mut_data()?;
    let config = bytemuck::from_bytes_mut::<Config>(&mut data);

    config.account_type = AccountType::Config;
    config.authority = OptionalNonZeroPubkey(*ctx.accounts.config_authority.key);
    config.slash_authority = OptionalNonZeroPubkey(*ctx.accounts.slash_authority.key);
    config.vault_token = *ctx.accounts.vault_token.key;
    config.vault_bump = bump;
    config.cooldown_time_seconds = cooldown_time.into();
    config.max_deactivation_basis_points = max_deactivation_basis_points.into();

    Ok(())
}
