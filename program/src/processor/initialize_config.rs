use paladin_rewards_program_client::accounts::HolderRewards;
use solana_program::{
    entrypoint::ProgramResult, msg, program_error::ProgramError, pubkey::Pubkey, rent::Rent,
    sysvar::Sysvar,
};
use spl_discriminator::SplDiscriminate;
use spl_pod::optional_keys::OptionalNonZeroPubkey;
use spl_token_2022::{
    extension::{
        transfer_hook::TransferHook, BaseStateWithExtensions, ExtensionType, PodStateWithExtensions,
    },
    pod::{PodAccount, PodCOption, PodMint},
};

use crate::{
    err,
    error::StakeError,
    instruction::accounts::{Context, InitializeConfigAccounts},
    require,
    state::{find_vault_pda, Config, MAX_BASIS_POINTS},
};

/// List of extensions that must be present in the vault token account.
const VALID_VAULT_TOKEN_EXTENSIONS: &[ExtensionType] = &[
    ExtensionType::ImmutableOwner,
    ExtensionType::TransferHookAccount,
];

/// Creates Stake config account which controls staking parameters.
///
/// ### Accounts:
///
///   0. `[w]` Stake config
///   1. `[ ]` Mint
///   2. `[ ]` Vault
pub fn process_initialize_config(
    program_id: &Pubkey,
    ctx: Context<InitializeConfigAccounts>,
    slash_authority: Pubkey,
    config_authority: Pubkey,
    cooldown_time_seconds: u64,
    max_deactivation_basis_points: u16,
    sync_rewards_lamports: u64,
) -> ProgramResult {
    // Accounts validation.

    // mint
    // - must be initialized (checked by unpack)
    // - have rewards transfer hook
    let mint_data = ctx.accounts.mint.try_borrow_data()?;
    let mint = PodStateWithExtensions::<PodMint>::unpack(&mint_data)?;

    // Ensure the mint is configured with the expected `TransferHook` extension.
    if let Ok(transfer_hook) = mint.get_extension::<TransferHook>() {
        let hook_program_id: Option<Pubkey> = transfer_hook.program_id.into();

        require!(
            hook_program_id == Some(paladin_rewards_program_client::ID),
            StakeError::InvalidTransferHookProgramId,
            "expected {:?}, found {:?}",
            Some(paladin_rewards_program_client::ID),
            hook_program_id
        );
    } else {
        return err!(StakeError::MissingTransferHook);
    }

    // vault (token account)
    // - must be initialized (checked by unpack)
    // - have the vault signer (PDA) as owner
    // - no close authority
    // - no delegate
    // - no other extension apart from `ImmutableOwner` and `TransferHookAccount`
    // - have the correct mint
    // - amount equal to 0
    let vault_data = ctx.accounts.vault.try_borrow_data()?;
    let vault = PodStateWithExtensions::<PodAccount>::unpack(&vault_data)?;
    let (vault_signer, signer_bump) = find_vault_pda(ctx.accounts.config.key, program_id);
    require!(
        vault.base.owner == vault_signer,
        StakeError::InvalidTokenOwner,
        "vault"
    );
    require!(
        vault.base.close_authority == PodCOption::none(),
        StakeError::CloseAuthorityNotNone,
        "vault"
    );
    require!(
        vault.base.delegate == PodCOption::none(),
        StakeError::DelegateNotNone,
        "vault"
    );
    vault
        .get_extension_types()?
        .iter()
        .try_for_each(|extension_type| {
            if !VALID_VAULT_TOKEN_EXTENSIONS.contains(extension_type) {
                msg!("Invalid token extension: {:?}", extension_type);
                return Err(StakeError::InvalidTokenAccountExtension);
            }
            Ok(())
        })?;
    require!(
        &vault.base.mint == ctx.accounts.mint.key,
        StakeError::InvalidMint,
        "vault"
    );
    let amount: u64 = vault.base.amount.into();
    require!(amount == 0, StakeError::AmountGreaterThanZero, "vault");

    // Vault (holder rewards).
    // - Must match derivation.
    // - Must be owned by the rewards program.
    // - Must be initialized.
    // - Must have no sponsor.
    let (vault_holder_rewards, _) = HolderRewards::find_pda(ctx.accounts.vault.key);
    require!(
        ctx.accounts.vault_holder_rewards.key == &vault_holder_rewards,
        ProgramError::InvalidSeeds,
        "vault holder rewards"
    );
    require!(
        ctx.accounts.vault_holder_rewards.owner == &paladin_rewards_program_client::ID,
        ProgramError::IllegalOwner,
        "vault holder rewards"
    );
    require!(
        ctx.accounts.vault_holder_rewards.data_len() == HolderRewards::LEN,
        ProgramError::UninitializedAccount,
        "vault holder rewards"
    );
    let holder_rewards =
        HolderRewards::from_bytes(&ctx.accounts.vault_holder_rewards.data.borrow()).unwrap();
    require!(
        holder_rewards.rent_sponsor == Pubkey::default(),
        StakeError::InvalidHolderRewards,
        "vault holder rewards"
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
        sync_rewards_lamports,
        vault_authority_bump: signer_bump,
        _padding: [0; 5],
    };

    Ok(())
}
