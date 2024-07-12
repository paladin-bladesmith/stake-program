use paladin_rewards::accounts::HolderRewards;
use solana_program::{entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey};
use spl_token_2022::{extension::PodStateWithExtensions, pod::PodAccount};

use crate::{
    error::StakeError,
    instruction::accounts::{Context, HarvestHolderRewardsAccounts},
    require,
    state::{calculate_eligible_rewards, create_vault_pda, find_stake_pda, Config, Stake},
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
/// 0. `[]` Config account
/// 1. `[w]` Stake account
/// 2. `[w]` Vault token account
/// 3. `[]` Holder rewards account for vault token account
/// 4. `[w]` Destination account for withdrawn lamports
/// 5. `[s]` Stake authority
/// 6. `[]` Vault authority, PDA with seeds `['token-owner', stake_config]`
/// 7. `[]` Stake token mint, to get total supply
/// 8. `[]` SPL Token program
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

    let config_data = ctx.accounts.config.try_borrow_data()?;
    let config = bytemuck::try_from_bytes::<Config>(&config_data)
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

    // stake authority
    // - must be a signer
    // - must match the authority on the stake account

    require!(
        ctx.accounts.stake_authority.is_signer,
        ProgramError::MissingRequiredSignature,
        "stake authority",
    );

    require!(
        ctx.accounts.stake_authority.key == &stake.authority,
        StakeError::InvalidAuthority,
        "stake authority",
    );

    // vault authority
    // - derivation must match

    let vault_signer = create_vault_pda(ctx.accounts.config.key, config.signer_bump, program_id)?;

    require!(
        ctx.accounts.vault_authority.key == &vault_signer,
        StakeError::InvalidAuthority,
        "vault authority",
    );

    // holder rewards (for the vault token account)
    // - owner must be the rewards program
    // - derivation must match

    require!(
        ctx.accounts.holder_rewards.owner == &paladin_rewards::ID,
        ProgramError::InvalidAccountOwner,
        "holder rewards",
    );

    let (derivation, _) = Pubkey::find_program_address(
        &["holder".as_bytes(), ctx.accounts.vault.key.as_ref()],
        &paladin_rewards::ID,
    );

    require!(
        ctx.accounts.holder_rewards.key == &derivation,
        ProgramError::InvalidSeeds,
        "holder rewards",
    );

    // Determine the holder rewards.

    let holder_rewards = HolderRewards::try_from(ctx.accounts.holder_rewards)?;
    let rewards = calculate_eligible_rewards(
        holder_rewards.last_accumulated_rewards_per_token,
        stake.last_seen_holder_rewards_per_token(),
        stake.amount,
    )?;
    // update the last seen holder rewards
    stake.set_last_seen_holder_rewards_per_token(holder_rewards.last_accumulated_rewards_per_token);

    // Withdraw the holder rewards to the destination account.
    //
    // Rewards are stored on the `vault` account.
    //
    // TODO: Need to withdraw the rewards to the destination account, but these are
    // currently stored on the vault token account. We could use `WithdrawExcessLamports`
    // and put back the excess lamports back into the vault token account.

    Ok(())
}
