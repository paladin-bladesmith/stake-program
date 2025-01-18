use bytemuck::Pod;
use paladin_rewards_program_client::accounts::HolderRewards;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program::invoke_signed,
    program_error::ProgramError, program_pack::IsInitialized, pubkey::Pubkey, rent::Rent,
    sysvar::Sysvar,
};
use spl_discriminator::{ArrayDiscriminator, SplDiscriminate};
use spl_token_2022::{extension::PodStateWithExtensions, instruction::burn_checked, pod::PodMint};

use crate::{
    error::StakeError,
    instruction::{
        accounts::{
            HarvestHolderRewardsAccounts, HarvestSolStakerRewardsAccounts,
            HarvestValidatorRewardsAccounts, InitializeConfigAccounts,
            InitializeSolStakerStakeAccounts, InitializeValidatorStakeAccounts,
            SetAuthorityAccounts, SlashSolStakerStakeAccounts, SlashValidatorStakeAccounts,
            SolStakerMoveTokensAccounts, SolStakerSetAuthorityOverrideAccounts,
            SolStakerStakeTokensAccounts, SolStakerSyncAuthorityAccounts, UnstakeTokensAccounts,
            UpdateConfigAccounts, ValidatorOverrideStakedLamportsAccounts,
            ValidatorStakeTokensAccounts, ValidatorSyncAuthorityAccounts,
        },
        StakeInstruction,
    },
    state::{
        calculate_eligible_rewards, calculate_maximum_stake_for_lamports_amount,
        calculate_stake_rewards_per_token, find_sol_staker_stake_pda, find_validator_stake_pda,
        Config, Delegation, SolStakerStake, ValidatorStake,
    },
};

mod harvest_holder_rewards;
mod harvest_sol_staker_rewards;
mod harvest_validator_rewards;
mod initialize_config;
mod initialize_sol_staker_stake;
mod initialize_validator_stake;
mod set_authority;
mod slash_sol_staker_stake;
mod slash_validator_stake;
mod sol_staker_move_tokens;
mod sol_staker_set_authority_override;
mod sol_staker_stake_tokens;
mod sol_staker_sync_authority;
mod unstake_tokens;
mod update_config;
mod validator_override_staked_lamports;
mod validator_stake_tokens;
mod validator_sync_authority;

#[inline(always)]
pub fn process_instruction<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = StakeInstruction::unpack(instruction_data)?;

    match instruction {
        StakeInstruction::HarvestHolderRewards => {
            msg!("Instruction: HarvestHolderRewards");
            harvest_holder_rewards::process_harvest_holder_rewards(
                program_id,
                HarvestHolderRewardsAccounts::context(accounts)?,
            )
        }
        StakeInstruction::HarvestValidatorRewards => {
            msg!("Instruction: HarvestValidatorRewards");
            harvest_validator_rewards::process_harvest_validator_rewards(
                program_id,
                HarvestValidatorRewardsAccounts::context(accounts)?,
            )
        }
        StakeInstruction::InitializeConfig {
            slash_authority,
            config_authority,
            cooldown_time_seconds,
            max_deactivation_basis_points,
            sync_rewards_lamports,
        } => {
            msg!("Instruction: InitializeConfig");
            initialize_config::process_initialize_config(
                program_id,
                InitializeConfigAccounts::context(accounts)?,
                slash_authority,
                config_authority,
                cooldown_time_seconds,
                max_deactivation_basis_points,
                sync_rewards_lamports,
            )
        }
        StakeInstruction::InitializeValidatorStake => {
            msg!("Instruction: InitializeValidatorStake");
            initialize_validator_stake::process_initialize_validator_stake(
                program_id,
                InitializeValidatorStakeAccounts::context(accounts)?,
            )
        }
        StakeInstruction::SetAuthority(authority) => {
            msg!("Instruction: SetAuthority");
            set_authority::process_set_authority(
                program_id,
                SetAuthorityAccounts::context(accounts)?,
                authority,
            )
        }
        StakeInstruction::SlashValidatorStake(amount) => {
            msg!("Instruction: SlashValidatorStake");
            slash_validator_stake::process_slash_validator_stake(
                program_id,
                SlashValidatorStakeAccounts::context(accounts)?,
                amount,
            )
        }
        StakeInstruction::ValidatorStakeTokens(amount) => {
            msg!("Instruction: ValidatorStakeTokens");
            validator_stake_tokens::process_validator_stake_tokens(
                program_id,
                ValidatorStakeTokensAccounts::context(accounts)?,
                amount,
            )
        }
        StakeInstruction::UpdateConfig(field) => {
            msg!("Instruction: UpdateConfig");
            update_config::process_update_config(
                program_id,
                UpdateConfigAccounts::context(accounts)?,
                field,
            )
        }
        StakeInstruction::InitializeSolStakerStake => {
            msg!("Instruction: InitializeSolStakerStake");
            initialize_sol_staker_stake::process_initialize_sol_staker_stake(
                program_id,
                InitializeSolStakerStakeAccounts::context(accounts)?,
            )
        }
        StakeInstruction::SolStakerStakeTokens(amount) => {
            msg!("Instruction: SolStakerStakeTokens");
            sol_staker_stake_tokens::process_sol_staker_stake_tokens(
                program_id,
                SolStakerStakeTokensAccounts::context(accounts)?,
                amount,
            )
        }
        StakeInstruction::HarvestSolStakerRewards => {
            msg!("Instruction: HarvestSolStakerRewards");
            harvest_sol_staker_rewards::process_harvest_sol_staker_rewards(
                program_id,
                HarvestSolStakerRewardsAccounts::context(accounts)?,
            )
        }
        StakeInstruction::UnstakeTokens { amount } => {
            msg!("Instruction: UnstakeTokens");
            unstake_tokens::process_unstake_tokens(
                program_id,
                UnstakeTokensAccounts::context(accounts)?,
                amount,
            )
        }
        StakeInstruction::SlashSolStakerStake(amount) => {
            msg!("Instruction: SlashSolStakerStake");
            slash_sol_staker_stake::process_slash_sol_staker_stake(
                program_id,
                SlashSolStakerStakeAccounts::context(accounts)?,
                amount,
            )
        }
        StakeInstruction::SolStakerMoveTokens { amount } => {
            msg!("Instruction: MoveSolStakerStake");
            sol_staker_move_tokens::process_sol_staker_move_tokens(
                program_id,
                SolStakerMoveTokensAccounts::context(accounts)?,
                amount,
            )
        }
        StakeInstruction::SolStakerSyncAuthority => {
            msg!("Instruction: SolStakerUpdateAuthority");
            sol_staker_sync_authority::process_sol_staker_sync_authority(
                program_id,
                SolStakerSyncAuthorityAccounts::context(accounts)?,
            )
        }
        StakeInstruction::SolStakerSetAuthorityOverride {
            authority_original,
            authority_override,
        } => {
            msg!("Instruction: SolStakerSetAuthorityOverride");
            sol_staker_set_authority_override::process_sol_staker_set_authority_override(
                program_id,
                SolStakerSetAuthorityOverrideAccounts::context(accounts)?,
                authority_original,
                authority_override,
            )
        }
        StakeInstruction::ValidatorOverrideStakedLamports { amount_min } => {
            msg!("Instruction: ValidatorOverrideStakedLamports");
            validator_override_staked_lamports::process_validator_override_staked_lamports(
                program_id,
                ValidatorOverrideStakedLamportsAccounts::context(accounts)?,
                amount_min,
            )
        }
        StakeInstruction::ValidatorSyncAuthority => {
            msg!("Instruction: ValidatorSyncAuthority");
            validator_sync_authority::process_validator_sync_authority(
                program_id,
                ValidatorSyncAuthorityAccounts::context(accounts)?,
            )
        }
    }
}

#[macro_export]
macro_rules! require {
    ( $constraint:expr, $error:expr $(,)? ) => {
        if !$constraint {
            return Err($error.into());
        }
    };
    ( $constraint:expr, $error:expr, $message:expr $(,)? ) => {
        if !$constraint {
            solana_program::msg!("Constraint failed: {}", $message);
            return Err($error.into());
        }
    };
    ( $constraint:expr, $error:expr, $message:literal, $($args:tt)+ ) => {
        require!( $constraint, $error, format!($message, $($args)+) );
    };
}

#[inline]
pub fn unpack_initialized<T: Pod + IsInitialized>(data: &[u8]) -> Result<&T, ProgramError> {
    let account =
        bytemuck::try_from_bytes::<T>(data).map_err(|_error| ProgramError::InvalidAccountData)?;

    require!(account.is_initialized(), ProgramError::UninitializedAccount);

    Ok(account)
}

/// Unpacks an initialized account from the given data and
/// returns a mutable reference to it.
#[inline]
pub fn unpack_initialized_mut<T: Pod + IsInitialized>(
    data: &mut [u8],
) -> Result<&mut T, ProgramError> {
    let account = bytemuck::try_from_bytes_mut::<T>(data)
        .map_err(|_error| ProgramError::InvalidAccountData)?;

    require!(account.is_initialized(), ProgramError::UninitializedAccount);

    Ok(account)
}

/// Unpacks the delegation information from either a `SolStakerStake` and `ValidatorStake`
/// accounts.
///
/// This function will validate that the account data is initialized and derivation matches
/// the expected PDA derivation.
#[inline]
pub fn unpack_delegation_mut_checked<'a>(
    stake_data: &'a mut [u8],
    stake: &Pubkey,
    config: &Pubkey,
    program_id: &Pubkey,
) -> Result<&'a mut Delegation, ProgramError> {
    let (delegation, derivation) = match &stake_data[..ArrayDiscriminator::LENGTH] {
        SolStakerStake::SPL_DISCRIMINATOR_SLICE => {
            let sol_staker = unpack_initialized_mut::<SolStakerStake>(stake_data)?;

            let (derivation, _) =
                find_sol_staker_stake_pda(&sol_staker.sol_stake, config, program_id);

            (&mut sol_staker.delegation, derivation)
        }
        ValidatorStake::SPL_DISCRIMINATOR_SLICE => {
            let validator = unpack_initialized_mut::<ValidatorStake>(stake_data)?;

            let (derivation, _) =
                find_validator_stake_pda(&validator.delegation.validator_vote, config, program_id);

            (&mut validator.delegation, derivation)
        }
        _ => return Err(ProgramError::InvalidAccountData),
    };

    require!(stake == &derivation, ProgramError::InvalidSeeds, "stake");

    Ok(delegation)
}

pub(crate) fn sync_config_lamports(
    config: &AccountInfo,
    config_state: &mut Config,
) -> ProgramResult {
    let lamport_delta = config
        .lamports()
        .checked_sub(config_state.lamports_last)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    let rewards_per_token =
        calculate_stake_rewards_per_token(lamport_delta, config_state.token_amount_effective)?;
    config_state.accumulated_stake_rewards_per_token =
        u128::from(config_state.accumulated_stake_rewards_per_token)
            .wrapping_add(rewards_per_token)
            .into();
    config_state.lamports_last = config.lamports();

    Ok(())
}

pub(crate) struct HarvestAccounts<'a, 'info> {
    pub(crate) config: &'a AccountInfo<'info>,
    pub(crate) vault_holder_rewards: &'a AccountInfo<'info>,
    pub(crate) authority: &'a AccountInfo<'info>,
}

pub(crate) fn harvest(
    accounts: HarvestAccounts,
    config_state: &mut Config,
    delegation: &mut Delegation,
    keeper: Option<&AccountInfo>,
) -> ProgramResult {
    // Provided authority must match expected.
    require!(
        accounts.authority.key == &delegation.authority,
        StakeError::InvalidAuthority,
        "authority"
    );

    // Sync the config accounts lamports.
    sync_config_lamports(accounts.config, config_state)?;

    // Compute the staking rewards.
    let staking_reward = calculate_eligible_rewards(
        config_state.accumulated_stake_rewards_per_token.into(),
        delegation.last_seen_stake_rewards_per_token.into(),
        delegation.effective_amount,
    )?;

    // Compute the holder reward.
    let (derivation, _) = HolderRewards::find_pda(&config_state.vault);
    require!(
        accounts.vault_holder_rewards.key == &derivation,
        ProgramError::InvalidSeeds,
        "holder rewards",
    );
    let vault_holder_rewards = HolderRewards::try_from(accounts.vault_holder_rewards)?;
    let holder_reward = calculate_eligible_rewards(
        vault_holder_rewards.last_accumulated_rewards_per_token,
        delegation.last_seen_holder_rewards_per_token.into(),
        delegation.staked_amount,
    )?;

    // Claim both at the same time.
    let total_reward = staking_reward
        .checked_add(holder_reward)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    delegation.last_seen_stake_rewards_per_token = config_state.accumulated_stake_rewards_per_token;
    delegation.last_seen_holder_rewards_per_token = vault_holder_rewards
        .last_accumulated_rewards_per_token
        .into();

    // Withdraw the lamports from the config account.
    let rent_exempt_minimum = Rent::get()?.minimum_balance(Config::LEN);
    let config_lamports = accounts
        .config
        .lamports()
        .checked_sub(total_reward)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    assert!(config_lamports >= rent_exempt_minimum);

    // Pay the keeper first, if one is set.
    let keeper_reward = match keeper {
        Some(keeper) => {
            let keeper_reward = std::cmp::min(total_reward, config_state.sync_rewards_lamports);
            let keeper_lamports = keeper
                .lamports()
                .checked_add(keeper_reward)
                .ok_or(ProgramError::ArithmeticOverflow)?;
            **keeper.try_borrow_mut_lamports()? = keeper_lamports;

            keeper_reward
        }
        None => 0,
    };

    // Pay the delegator.
    let delegator_reward = total_reward
        .checked_sub(keeper_reward)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    let recipient_lamports = accounts
        .authority
        .lamports()
        .checked_add(delegator_reward)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    // Update the lamport amounts.
    config_state.lamports_last = config_lamports;
    **accounts.config.try_borrow_mut_lamports()? = config_lamports;
    **accounts.authority.try_borrow_mut_lamports()? = recipient_lamports;

    Ok(())
}

pub(crate) fn sync_effective(
    config: &mut Config,
    delegation: &mut Delegation,
    (lamports_stake, lamports_stake_min): (u64, u64),
) -> ProgramResult {
    let lamports_stake = std::cmp::max(lamports_stake, lamports_stake_min);
    let limit = calculate_maximum_stake_for_lamports_amount(lamports_stake)?;
    let new_effective_amount = std::cmp::min(delegation.staked_amount, limit);

    // Update states.
    config.token_amount_effective = config
        .token_amount_effective
        .checked_sub(delegation.effective_amount)
        .ok_or(ProgramError::ArithmeticOverflow)?
        .checked_add(new_effective_amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    delegation.effective_amount = new_effective_amount;

    Ok(())
}

/// Arguments to process the slash of a stake delegation.
struct SlashArgs<'a, 'b> {
    delegation: &'b mut Delegation,
    mint_info: &'b AccountInfo<'a>,
    vault_info: &'b AccountInfo<'a>,
    vault_authority_info: &'b AccountInfo<'a>,
    token_program_info: &'b AccountInfo<'a>,
    signer_seeds: &'b [&'b [u8]],
    amount: u64,
}

/// Processes the slash for a stake delegation.
fn process_slash_for_delegation(args: SlashArgs) -> ProgramResult {
    let SlashArgs {
        delegation,
        mint_info,
        vault_info,
        vault_authority_info,
        token_program_info,
        signer_seeds,
        amount,
    } = args;

    require!(
        token_program_info.key == &spl_token_2022::ID,
        ProgramError::IncorrectProgramId,
        "token program"
    );

    // Update the stake amount on both stake and config accounts:
    //
    //   1. the amount slashed is taken from the stake amount (this includes
    //      the amount that is currently staked + deactivating);
    //
    //   2. if not enough, the remaining is ignored and the stake account is
    //      left with 0 amount;
    //
    //   3. if the stake account has deactivating tokens, make sure that the
    //      deactivating amount is at least the same as the stake amount (it might
    //      need to be reduced due to slashing).

    require!(
        amount > 0,
        StakeError::InvalidAmount,
        "amount must be greater than 0"
    );

    // Compute actual slash & new stake numbers.
    let actual_slash = std::cmp::min(amount, delegation.staked_amount);
    let staked_amount = delegation
        .staked_amount
        .checked_sub(actual_slash)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    // NB: Effective is updated by the caller via `sync_effective`.

    // Update stake amounts.
    delegation.staked_amount = staked_amount;

    // Burn the tokens from the vault account (if there are tokens to slash).
    if actual_slash > 0 {
        let mint_data = mint_info.try_borrow_data()?;
        let mint = PodStateWithExtensions::<PodMint>::unpack(&mint_data)?;
        let decimals = mint.base.decimals;

        drop(mint_data);
        let burn_ix = burn_checked(
            token_program_info.key,
            vault_info.key,
            mint_info.key,
            vault_authority_info.key,
            &[],
            actual_slash,
            decimals,
        )?;
        invoke_signed(
            &burn_ix,
            &[
                token_program_info.clone(),
                vault_info.clone(),
                mint_info.clone(),
                vault_authority_info.clone(),
            ],
            &[signer_seeds],
        )?;
    }

    Ok(())
}
