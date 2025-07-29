use solana_program::{
    entrypoint::ProgramResult, program::invoke_signed, program_error::ProgramError,
    program_option::COption, program_pack::Pack, pubkey::Pubkey, rent::Rent, system_instruction,
    sysvar::Sysvar,
};
use spl_discriminator::SplDiscriminate;
use spl_pod::optional_keys::OptionalNonZeroPubkey;
use spl_token::state::{Account as TokenAccount, Mint};

use crate::{
    error::StakeError,
    instruction::accounts::{Context, InitializeConfigAccounts},
    require,
    state::{find_vault_pda, get_vault_pda_signer_seeds, Config, MAX_BASIS_POINTS},
};

/// Creates Stake config account which controls staking parameters.
#[allow(clippy::too_many_arguments)]
pub fn process_initialize_config(
    program_id: &Pubkey,
    ctx: Context<InitializeConfigAccounts>,
    slash_authority: Pubkey,
    config_authority: Pubkey,
    cooldown_time_seconds: u64,
    max_deactivation_basis_points: u16,
    sync_rewards_lamports: u64,
    duna_document_hash: [u8; 32],
) -> ProgramResult {
    // Accounts validation.

    // mint
    // - must be initialized (checked by unpack)
    // - have rewards transfer hook
    let mint_data = ctx.accounts.mint.try_borrow_data()?;
    Mint::unpack(&mint_data)?;

    // Verify vault PDA
    let (vault_signer, signer_bump) = find_vault_pda(ctx.accounts.config.key, program_id);
    let signer_bump = [signer_bump];
    let vault_seeds = get_vault_pda_signer_seeds(ctx.accounts.config.key, &signer_bump);

    // Ensure the provided vault pda address is the correct address
    require!(
        ctx.accounts.vault_pda.key == &vault_signer,
        StakeError::IncorrectVaultPdaAccount,
        "vault pda"
    );

    // Confirm vault PDA account is empty
    require!(
        ctx.accounts.vault_pda.data_is_empty(),
        ProgramError::AccountAlreadyInitialized,
        "vault pda"
    );

    invoke_signed(
        &system_instruction::assign(&vault_signer, program_id),
        &[ctx.accounts.vault_pda.clone()],
        &[&vault_seeds],
    )?;

    // vault (token account)
    // - must be initialized (checked by unpack)
    // - have the vault signer (PDA) as owner
    // - no close authority
    // - no delegate
    // - have the correct mint
    // - amount equal to 0
    let vault_data = ctx.accounts.vault.try_borrow_data()?;
    let vault = TokenAccount::unpack(&vault_data)?;
    require!(
        vault.owner == vault_signer,
        StakeError::InvalidTokenOwner,
        "vault"
    );
    require!(
        vault.close_authority == COption::None,
        StakeError::CloseAuthorityNotNone,
        "vault"
    );
    require!(
        vault.delegate == COption::None,
        StakeError::DelegateNotNone,
        "vault"
    );
    require!(
        &vault.mint == ctx.accounts.mint.key,
        StakeError::InvalidMint,
        "vault"
    );
    require!(
        vault.amount == 0,
        StakeError::AmountGreaterThanZero,
        "vault"
    );

    // config
    // - owner must be this program
    // - have the correct length
    // - be uninitialized
    // - be rent exempt
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
        config.is_uninitialized(),
        ProgramError::AccountAlreadyInitialized,
        "config"
    );
    require!(
        ctx.accounts.config.lamports() >= Rent::get()?.minimum_balance(Config::LEN),
        ProgramError::AccountNotRentExempt,
        "config",
    );

    // Validate the maximum deactivation basis points argument.
    require!(
        max_deactivation_basis_points <= MAX_BASIS_POINTS as u16,
        ProgramError::InvalidArgument,
        "basis points exceeds maximum allowed value of {}",
        MAX_BASIS_POINTS
    );

    // Create the vault holder rewards account
    invoke_signed(
        &paladin_rewards_program_client::instructions::InitializeHolderRewards {
            // NB: Account correctness validated by paladin rewards program.
            holder_rewards_pool: *ctx.accounts.holder_rewards_pool.key,
            holder_rewards_pool_token_account_info: *ctx
                .accounts
                .holder_rewards_pool_token_account
                .key,
            owner: *ctx.accounts.vault_pda.key,
            holder_rewards: *ctx.accounts.vault_holder_rewards.key,
            mint: *ctx.accounts.mint.key,
            system_program: *ctx.accounts.system_program.key,
        }
        .instruction(),
        &[
            ctx.accounts.holder_rewards_pool.clone(),
            ctx.accounts.holder_rewards_pool_token_account.clone(),
            ctx.accounts.vault_pda.clone(),
            ctx.accounts.vault_holder_rewards.clone(),
            ctx.accounts.mint.clone(),
            ctx.accounts.system_program.clone(),
            ctx.accounts.rewards_program.clone(),
        ],
        &[&vault_seeds],
    )?;

    // Initialize the stake config account.
    *config = Config {
        discriminator: Config::SPL_DISCRIMINATOR.into(),
        authority: OptionalNonZeroPubkey(config_authority),
        slash_authority: OptionalNonZeroPubkey(slash_authority),
        vault: *ctx.accounts.vault.key,
        cooldown_time_seconds,
        token_amount_effective: 0,
        lamports_last: ctx.accounts.config.lamports(),
        accumulated_stake_rewards_per_token: 0.into(),
        max_deactivation_basis_points,
        duna_document_hash,
        sync_rewards_lamports,
        vault_authority_bump: signer_bump[0],
        _padding: [0; 5],
    };

    Ok(())
}
