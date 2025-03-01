use solana_program::{entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey};
use spl_token_2022::{extension::PodStateWithExtensions, pod::PodAccount};

use crate::{
    err,
    error::StakeError,
    instruction::accounts::{Context, SlashStakeAccounts},
    processor::{
        harvest, process_slash_for_delegation, sync_effective, unpack_initialized_mut,
        unpack_stake, HarvestAccounts, SlashArgs,
    },
    require,
    state::{create_vault_pda, get_vault_pda_signer_seeds, Config},
};

/// Slashes stake account for the given amount.
///
/// Burns the given amount of tokens from the vault account, and reduces the
/// amount in the stake account.
///
/// 0. `[w]` Config
/// 1. `[w]` Stake
/// 1. `[w]` Stake authority
/// 3. `[s]` Slash authority
/// 4. `[w]` Stake token mint
/// 5. `[w]` Vault token account
/// 6. `[ ]` Vault holder rewards
/// 7. `[ ]` Vault authority
/// 8. `[ ]` Token program
///
/// Instruction data: amount of tokens to slash.
pub fn process_slash_stake(
    program_id: &Pubkey,
    ctx: Context<SlashStakeAccounts>,
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
    let mut config = ctx.accounts.config.data.borrow_mut();
    let config = unpack_initialized_mut::<Config>(&mut config)?;

    // stake
    // - owner must be the stake program
    // - must be a ValidatorStake or SolStakerStake account
    // - must be initialized
    // - must have the correct derivation
    require!(
        ctx.accounts.stake.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "stake"
    );
    let mut stake_borrow = &mut ctx.accounts.stake.try_borrow_mut_data()?;
    let (derivation, lamports, lamports_min, delegation) =
        unpack_stake(program_id, ctx.accounts.config.key, &mut stake_borrow)?;
    require!(
        ctx.accounts.stake.key == &derivation,
        ProgramError::InvalidSeeds,
        "stake",
    );

    // Harvest rewards & update last claim tracking.
    harvest(
        HarvestAccounts {
            config: ctx.accounts.config,
            vault_holder_rewards: ctx.accounts.vault_holder_rewards,
            authority: ctx.accounts.stake_authority,
        },
        config,
        delegation,
        None,
    )?;

    // slash authority
    // - must be a signer
    // - must match the slash authority on the config account
    //
    // When there is no slash authority set, the stake account cannot be slashed and
    // an error is returned.
    let Some(slash_authority) = Option::<Pubkey>::from(config.slash_authority) else {
        return err!(StakeError::AuthorityNotSet, "slash authority");
    };
    require!(
        ctx.accounts.slash_authority.key == &slash_authority,
        StakeError::InvalidAuthority,
        "slash authority",
    );
    require!(
        ctx.accounts.slash_authority.is_signer,
        ProgramError::MissingRequiredSignature,
        "slash authority",
    );

    // vault authority
    // - derivation must match
    let signer_bump = [config.vault_authority_bump];
    let derivation = create_vault_pda(ctx.accounts.config.key, &signer_bump, program_id)?;
    require!(
        ctx.accounts.vault_authority.key == &derivation,
        StakeError::InvalidAuthority,
        "vault authority",
    );
    let signer_seeds = get_vault_pda_signer_seeds(ctx.accounts.config.key, &signer_bump);

    // vault
    // - must be the token account on the stake config account
    require!(
        ctx.accounts.vault.key == &config.vault,
        StakeError::IncorrectVaultAccount,
    );
    let vault_data = ctx.accounts.vault.try_borrow_data()?;
    let vault = PodStateWithExtensions::<PodAccount>::unpack(&vault_data)?;

    // mint
    // - must match the stake vault mint
    require!(
        &vault.base.mint == ctx.accounts.mint.key,
        StakeError::InvalidMint,
        "mint"
    );

    // Process the slash for the stake delegation.
    //
    // This will burn the given amount of tokens from the vault account, and
    // update the stake delegation on the stake and config accounts.
    drop(vault_data);
    process_slash_for_delegation(SlashArgs {
        delegation: delegation,
        mint_info: ctx.accounts.mint,
        vault_info: ctx.accounts.vault,
        vault_authority_info: ctx.accounts.vault_authority,
        token_program_info: ctx.accounts.token_program,
        amount,
        signer_seeds: &signer_seeds,
    })?;

    // Sync the new effective stake.
    sync_effective(config, delegation, (lamports, lamports_min))?;

    Ok(())
}
