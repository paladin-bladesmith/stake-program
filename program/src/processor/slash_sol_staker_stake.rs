use solana_program::{entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey};
use spl_token_2022::{extension::PodStateWithExtensions, pod::PodAccount};

use crate::{
    err,
    error::StakeError,
    instruction::accounts::{Context, SlashSolStakerStakeAccounts},
    processor::{process_slash_for_delegation, unpack_initialized_mut, SlashArgs},
    require,
    state::{
        create_vault_pda, find_sol_staker_stake_pda, find_validator_stake_pda,
        get_vault_pda_signer_seeds, Config, SolStakerStake, ValidatorStake,
    },
};

/// Slashes a SOL staker stake account for the given amount.
///
/// Burns the given amount of tokens from the vault account, and reduces the
/// amount in the stake account.
///
/// 0. `[w]` Config account
/// 1. `[w]` SOL staker stake account
/// 2. `[w]` Validator stake account
/// 3. `[s]` Slash authority
/// 4. `[w]` Vault token account
/// 5. `[w]` Stake Token Mint
/// 6. `[ ]` Vault authority, PDA with seeds `['token-owner', stake_config]`
/// 7. `[ ]` SPL Token program
///
/// Instruction data: amount of tokens to slash.
pub fn process_slash_sol_staker_stake(
    program_id: &Pubkey,
    ctx: Context<SlashSolStakerStakeAccounts>,
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
    // - must be a SolStakerStake account
    // - must be initialized
    // - derivation must match (validates the config account)

    require!(
        ctx.accounts.stake.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "stake"
    );

    let mut stake_data = ctx.accounts.stake.try_borrow_mut_data()?;
    let stake = unpack_initialized_mut::<SolStakerStake>(&mut stake_data)?;

    let (derivation, _) =
        find_sol_staker_stake_pda(&stake.sol_stake, ctx.accounts.config.key, program_id);

    require!(
        ctx.accounts.stake.key == &derivation,
        ProgramError::InvalidSeeds,
        "stake",
    );

    // validator stake
    // - owner must be the stake program
    // - must be a ValidatorStake account
    // - must be initialized
    // - derivation must match (validates the config account)

    require!(
        ctx.accounts.validator_stake.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "validator stake"
    );

    let mut stake_data = ctx.accounts.validator_stake.try_borrow_mut_data()?;
    let validator_stake = unpack_initialized_mut::<ValidatorStake>(&mut stake_data)?;

    let (derivation, _) = find_validator_stake_pda(
        // validates the sol staker stake <-> validator stake relationship
        &stake.delegation.validator_vote,
        ctx.accounts.config.key,
        program_id,
    );

    require!(
        ctx.accounts.validator_stake.key == &derivation,
        ProgramError::InvalidSeeds,
        "validator stake",
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
    //
    // This will burn the given amount of tokens from the vault account, and
    // update the stake delegation on the stake and config accounts.

    let slashed_amount = process_slash_for_delegation(SlashArgs {
        config,
        delegation: &mut stake.delegation,
        mint_info: ctx.accounts.mint,
        vault_info: ctx.accounts.vault,
        vault_authority_info: ctx.accounts.vault_authority,
        token_program_info: ctx.accounts.token_program,
        amount,
        signer_seeds: &signer_seeds,
    })?;

    // Decreases the slashed amount on the corresponding validator stake account.

    validator_stake.total_staked_token_amount = validator_stake
        .total_staked_token_amount
        .checked_sub(slashed_amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    Ok(())
}
