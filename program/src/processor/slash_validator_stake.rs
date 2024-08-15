use solana_program::{entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey};
use spl_token_2022::{extension::PodStateWithExtensions, pod::PodAccount};

use crate::{
    err,
    error::StakeError,
    instruction::accounts::{Context, SlashValidatorStakeAccounts},
    processor::{process_slash_for_delegation, unpack_initialized_mut, SlashArgs},
    require,
    state::{
        create_vault_pda, find_validator_stake_pda, get_vault_pda_signer_seeds, Config,
        ValidatorStake,
    },
};

/// Slashes a validator stake account for the given amount
///
/// Burns the given amount of tokens from the vault account, and reduces the
/// amount in the stake account.
///
/// 0. `[w]` Config account
/// 1. `[w]` Validator stake account
/// 2. `[s]` Slash authority
/// 3. `[w]` Vault token account
/// 4. `[]` Stake Token Mint
/// 5. `[]` Vault authority, PDA with seeds `['token-owner', stake_config]`
/// 6. `[]` SPL Token program
///
/// Instruction data: amount of tokens to slash
pub fn process_slash_validator_stake(
    program_id: &Pubkey,
    ctx: Context<SlashValidatorStakeAccounts>,
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

    // stake
    // - owner must be the stake program
    // - must be a ValidatorStake account
    // - must be initialized
    // - derivation must match (validates the config account)

    require!(
        ctx.accounts.stake.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "stake"
    );

    let mut stake_data = ctx.accounts.stake.try_borrow_mut_data()?;
    let stake = unpack_initialized_mut::<ValidatorStake>(&mut stake_data)?;

    let (derivation, _) = find_validator_stake_pda(
        &stake.delegation.validator_vote,
        ctx.accounts.config.key,
        program_id,
    );

    require!(
        ctx.accounts.stake.key == &derivation,
        ProgramError::InvalidSeeds,
        "stake",
    );

    // slash authority
    // - must be a signer
    // - must match the slash authority on the config account
    //
    // When there is no slash authority set, the stake account cannot be slashed and
    // an error is returned.

    if let Some(slash_authority) = Option::<Pubkey>::from(config.slash_authority) {
        require!(
            ctx.accounts.slash_authority.key == &slash_authority,
            StakeError::InvalidAuthority,
            "slash authority",
        );
    } else {
        return err!(StakeError::AuthorityNotSet, "slash authority");
    }

    require!(
        ctx.accounts.slash_authority.is_signer,
        ProgramError::MissingRequiredSignature,
        "stake authority",
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

    // Process the slash for the stake delegation.

    process_slash_for_delegation(SlashArgs {
        config,
        delegation: &mut stake.delegation,
        mint_info: ctx.accounts.mint,
        vault_info: ctx.accounts.vault,
        vault_authority_info: ctx.accounts.vault_authority,
        token_program_info: ctx.accounts.token_program,
        amount,
        signer_seeds: &signer_seeds,
    })?;

    Ok(())
}
