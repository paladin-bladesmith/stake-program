use paladin_rewards_program_client::accounts::HolderRewards;
use solana_program::{
    entrypoint::ProgramResult, msg, program::invoke_signed, program_error::ProgramError,
    pubkey::Pubkey, system_instruction,
};
use spl_token_2022::{
    extension::PodStateWithExtensions, instruction::withdraw_excess_lamports, pod::PodAccount,
};

use crate::{
    error::StakeError,
    instruction::accounts::{Context, HarvestHolderRewardsAccounts},
    processor::unpack_delegation_mut,
    require,
    state::{calculate_eligible_rewards, create_vault_pda, get_vault_pda_signer_seeds, Config},
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
/// 7. `[]` Stake token mint
/// 8. `[]` SPL Token program
/// 9. `[]` System program
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

    let stake_data = &mut ctx.accounts.stake.try_borrow_mut_data()?;
    // checks that the stake account is initialized and has the correct derivation
    let mut delegation = unpack_delegation_mut(
        stake_data,
        ctx.accounts.stake.key,
        ctx.accounts.config.key,
        program_id,
    )?;

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

    drop(vault_data);

    // stake authority
    // - must be a signer
    // - must match the authority on the stake account

    require!(
        ctx.accounts.stake_authority.is_signer,
        ProgramError::MissingRequiredSignature,
        "stake authority",
    );

    require!(
        ctx.accounts.stake_authority.key == &delegation.authority,
        StakeError::InvalidAuthority,
        "stake authority",
    );

    // vault authority
    // - derivation must match

    let signer_bump = [config.vault_authority_bump];
    let vault_signer = create_vault_pda(ctx.accounts.config.key, &signer_bump, program_id)?;

    require!(
        ctx.accounts.vault_authority.key == &vault_signer,
        StakeError::InvalidAuthority,
        "vault authority",
    );

    // destination
    // - must be different than the vault authority

    require!(
        ctx.accounts.destination.key != ctx.accounts.vault_authority.key,
        StakeError::InvalidDestinationAccount,
    );

    // holder rewards (for the vault token account)
    // - owner must be the rewards program
    // - derivation must match

    require!(
        ctx.accounts.holder_rewards.owner == &paladin_rewards_program_client::ID,
        ProgramError::InvalidAccountOwner,
        "holder rewards",
    );

    let (derivation, _) = HolderRewards::find_pda(ctx.accounts.vault.key);

    require!(
        ctx.accounts.holder_rewards.key == &derivation,
        ProgramError::InvalidSeeds,
        "holder rewards",
    );

    // Determine the holder rewards.

    let holder_rewards = HolderRewards::try_from(ctx.accounts.holder_rewards)?;
    let rewards = calculate_eligible_rewards(
        holder_rewards.last_accumulated_rewards_per_token,
        delegation.last_seen_holder_rewards_per_token.into(),
        delegation.amount,
    )?;

    // Withdraw the holder rewards to the destination account.

    if rewards != 0 {
        // update the last seen holder rewards
        delegation.last_seen_holder_rewards_per_token =
            holder_rewards.last_accumulated_rewards_per_token.into();

        // Rewards are stored on the `vault` token account. We need to first withdraw the excess lamports
        // from the vault token account to the vault authority account. Then transfer the rewards amount
        // to the destination account and send the remaining excess lamports back to the vault token
        // account.

        let withdraw_ix = withdraw_excess_lamports(
            ctx.accounts.token_program.key,
            ctx.accounts.vault.key,
            ctx.accounts.vault_authority.key,
            ctx.accounts.vault_authority.key,
            &[],
        )?;

        let signer_seeds = get_vault_pda_signer_seeds(ctx.accounts.config.key, &signer_bump);
        // Stores the starting lamports of the vault authority account before the withdraw so
        // we can calculate the amount of rewards withdrawn.
        let vault_authority_starting_lamports = ctx.accounts.vault_authority.lamports();

        invoke_signed(
            &withdraw_ix,
            &[
                ctx.accounts.token_program.clone(),
                ctx.accounts.vault.clone(),
                ctx.accounts.vault_authority.clone(),
            ],
            &[&signer_seeds],
        )?;

        // If the withdraw does not result in enough lamports to cover the rewards, only
        // harvest the available lamports. This should never happen, but the check is a
        // failsafe.
        let rewards = std::cmp::min(
            rewards,
            ctx.accounts
                .vault_authority
                .lamports()
                .saturating_sub(vault_authority_starting_lamports),
        );

        // Move the rewards amount from the vault authority to the destination account and
        // the remaining lamports back to the vault token account.
        invoke_signed(
            &system_instruction::transfer(
                ctx.accounts.vault_authority.key,
                ctx.accounts.destination.key,
                rewards,
            ),
            &[
                ctx.accounts.vault_authority.clone(),
                ctx.accounts.destination.clone(),
            ],
            &[&signer_seeds],
        )?;

        // Calculate the remaining lamports after the transfer.
        let remaining = ctx
            .accounts
            .vault_authority
            .lamports()
            .checked_sub(vault_authority_starting_lamports)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        invoke_signed(
            &system_instruction::transfer(
                ctx.accounts.vault_authority.key,
                ctx.accounts.vault.key,
                remaining,
            ),
            &[
                ctx.accounts.vault_authority.clone(),
                ctx.accounts.vault.clone(),
            ],
            &[&signer_seeds],
        )?;
    } else {
        msg!("No rewards to harvest");
    }

    Ok(())
}
