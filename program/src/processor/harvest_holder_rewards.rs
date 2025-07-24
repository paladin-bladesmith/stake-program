use paladin_rewards_program_client::accounts::HolderRewards;
use solana_program::{
    entrypoint::ProgramResult, program::invoke_signed, program_error::ProgramError,
    program_pack::Pack, pubkey::Pubkey,
};
use spl_token::state::Account as TokenAccount;

use crate::{
    error::StakeError,
    instruction::accounts::{Context, HarvestHolderRewardsAccounts},
    processor::{sync_config_lamports, transfer_excess_lamports, unpack_initialized_mut},
    require,
    state::{get_vault_pda_signer_seeds, Config},
};

/// Harvests holder SOL rewards earned by the given stake account.
///
/// Rewards are added to the vault token account.
///
/// NOTE: This mostly replicates the logic in the rewards program. Since the
/// staked tokens are all held by this program, stakers need a way to access
/// their portion of holder rewards.
///
/// This instruction requires that `unclaimed_rewards` be equal to `0` in
/// the token vault account. For ease of use, be sure to call the
/// `HarvestRewards` on the vault account before this.
///
/// 0. `[w]` Config account
/// 1. `[w]` Holder rewards pool
/// 2. `[w]` Vault token account
/// 3. `[w]` Vault holder rewards
/// 4. `[w]` Vault authority
/// 5. `[ ]` Stake token mint
/// 6. `[ ]` Token program
/// 7. `[ ]` Paladin rewards program
/// 8. `[ ]` System program
pub fn process_harvest_holder_rewards(
    program_id: &Pubkey,
    ctx: Context<HarvestHolderRewardsAccounts>,
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
    let config = unpack_initialized_mut::<Config>(&mut config_data)?;

    // vault
    // - must be the token account on the stake config account
    require!(
        ctx.accounts.vault.key == &config.vault,
        StakeError::IncorrectVaultAccount,
    );
    let vault_data = ctx.accounts.vault.try_borrow_data()?;
    let vault = TokenAccount::unpack(&vault_data)?;

    // mint
    // - must match the stake vault mint
    require!(
        &vault.mint == ctx.accounts.mint.key,
        StakeError::InvalidMint,
        "mint"
    );

    // vault authority
    // - derivation must match
    let signer_bump = [config.vault_authority_bump];
    let vault_seeds = get_vault_pda_signer_seeds(ctx.accounts.config.key, &signer_bump);
    let vault_signer = Pubkey::create_program_address(&vault_seeds, program_id)?;
    require!(
        ctx.accounts.vault_authority.key == &vault_signer,
        StakeError::InvalidAuthority,
        "vault authority",
    );

    // holder rewards (for the vault token account)
    // - owner must be the rewards program
    // - derivation must match
    require!(
        ctx.accounts.vault_holder_rewards.owner == &paladin_rewards_program_client::ID,
        ProgramError::InvalidAccountOwner,
        "holder rewards",
    );
    let (derivation, _) = HolderRewards::find_pda(ctx.accounts.vault_authority.key);
    require!(
        ctx.accounts.vault_holder_rewards.key == &derivation,
        ProgramError::InvalidSeeds,
        "holder rewards",
    );

    // Update the config's last seen lamports.
    sync_config_lamports(ctx.accounts.config, config)?;

    // Harvest latest holder rewards.
    drop(vault_data);
    drop(config_data);
    invoke_signed(
        &paladin_rewards_program_client::instructions::HarvestRewards {
            // NB: Account correctness validated by paladin rewards program.
            holder_rewards_pool: *ctx.accounts.holder_rewards_pool.key,
            holder_rewards_pool_token_account_info: *ctx
                .accounts
                .holder_rewards_pool_token_account
                .key,
            holder_rewards: *ctx.accounts.vault_holder_rewards.key,
            mint: *ctx.accounts.mint.key,
            owner: *ctx.accounts.vault_authority.key,
        }
        .instruction(),
        &[
            ctx.accounts.holder_rewards_pool.clone(),
            ctx.accounts.holder_rewards_pool_token_account.clone(),
            ctx.accounts.vault_holder_rewards.clone(),
            ctx.accounts.mint.clone(),
            ctx.accounts.vault_authority.clone(),
        ],
        &[&vault_seeds],
    )?;

    // Withdraw the excess lamports from the vault authority to config.
    transfer_excess_lamports(ctx.accounts.vault_authority, ctx.accounts.config)?;

    // Update the configs last seen lamports again.
    let mut config = ctx.accounts.config.try_borrow_mut_data()?;
    let config = unpack_initialized_mut::<Config>(&mut config)?;
    config.lamports_last = ctx.accounts.config.lamports();

    Ok(())
}
