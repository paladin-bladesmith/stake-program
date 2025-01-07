use solana_program::{entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey};

use crate::{
    error::StakeError,
    instruction::accounts::{Context, SolStakerMoveTokensAccounts},
    processor::{harvest, sync_effective, unpack_initialized_mut, HarvestAccounts},
    require,
    state::{find_sol_staker_stake_pda, Config, SolStakerStake},
};

pub(crate) fn process_sol_staker_move_tokens(
    program_id: &Pubkey,
    ctx: Context<SolStakerMoveTokensAccounts>,
    amount: u64,
) -> ProgramResult {
    // Config
    // - Owner must be this program
    // - Must be initialized
    require!(
        ctx.accounts.config.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "config"
    );
    let mut config = ctx.accounts.config.data.borrow_mut();
    let config = unpack_initialized_mut::<Config>(&mut config)?;

    // Sol staker authority.
    // - Must be signer.
    // - Must be authority on both stake accounts.
    require!(
        ctx.accounts.sol_staker_authority.is_signer,
        ProgramError::MissingRequiredSignature,
        "sol staker authority"
    );

    // Source sol staker stake
    // - Owner must be the stake program.
    // - Must be initialized.
    // - Must have the correct derivation (validates the config account).
    require!(
        ctx.accounts.source_sol_staker_stake.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "source sol staker stake"
    );
    let mut source_sol_staker_stake_data =
        ctx.accounts.source_sol_staker_stake.try_borrow_mut_data()?;
    let source_sol_staker_stake =
        unpack_initialized_mut::<SolStakerStake>(&mut source_sol_staker_stake_data)?;
    let (derivation, _) = find_sol_staker_stake_pda(
        &source_sol_staker_stake.sol_stake,
        ctx.accounts.config.key,
        program_id,
    );
    require!(
        ctx.accounts.source_sol_staker_stake.key == &derivation,
        ProgramError::InvalidSeeds,
        "source sol staker stake",
    );
    require!(
        &source_sol_staker_stake.delegation.authority == ctx.accounts.sol_staker_authority.key,
        StakeError::InvalidAuthority,
        "sol staker authority"
    );

    harvest(
        HarvestAccounts {
            config: ctx.accounts.config,
            vault_holder_rewards: ctx.accounts.vault_holder_rewards,
            authority: ctx.accounts.sol_staker_authority,
        },
        config,
        &mut source_sol_staker_stake.delegation,
        None,
    )?;

    // Destination sol staker stake
    // - Owner must be the stake program.
    // - Must be initialized.
    // - Must have the correct derivation (validates the config account).
    require!(
        ctx.accounts.destination_sol_staker_stake.owner == program_id,
        ProgramError::InvalidAccountOwner,
        "destination sol staker stake"
    );
    let mut destination_sol_staker_stake_data = ctx
        .accounts
        .destination_sol_staker_stake
        .try_borrow_mut_data()?;
    let destination_sol_staker_stake =
        unpack_initialized_mut::<SolStakerStake>(&mut destination_sol_staker_stake_data)?;
    let (derivation, _) = find_sol_staker_stake_pda(
        &destination_sol_staker_stake.sol_stake,
        ctx.accounts.config.key,
        program_id,
    );
    require!(
        ctx.accounts.destination_sol_staker_stake.key == &derivation,
        ProgramError::InvalidSeeds,
        "destination sol staker stake",
    );

    harvest(
        HarvestAccounts {
            config: ctx.accounts.config,
            vault_holder_rewards: ctx.accounts.vault_holder_rewards,
            authority: ctx.accounts.sol_staker_authority,
        },
        config,
        &mut destination_sol_staker_stake.delegation,
        None,
    )?;

    // Ensure authorities match (technically harvest should already prove this).
    assert_eq!(
        source_sol_staker_stake.delegation.authority,
        destination_sol_staker_stake.delegation.authority
    );

    // Decrease the staked balance of the source.
    source_sol_staker_stake.delegation.active_amount = source_sol_staker_stake
        .delegation
        .active_amount
        .checked_sub(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    // Increase the staked balance of the destination.
    destination_sol_staker_stake.delegation.active_amount = destination_sol_staker_stake
        .delegation
        .active_amount
        .checked_add(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    // Synchronize both delegation's new effective amounts.
    sync_effective(
        config,
        &mut source_sol_staker_stake.delegation,
        (source_sol_staker_stake.lamports_amount, 0),
    )?;
    sync_effective(
        config,
        &mut destination_sol_staker_stake.delegation,
        (destination_sol_staker_stake.lamports_amount, 0),
    )?;

    Ok(())
}
