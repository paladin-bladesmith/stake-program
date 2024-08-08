//! This code was AUTOGENERATED using the kinobi library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun kinobi to update it.
//!
//! <https://github.com/kinobi-so/kinobi>
//!

use borsh::BorshDeserialize;
use borsh::BorshSerialize;

/// Accounts.
pub struct HarvestSyncRewards {
    /// Stake config account
    pub config: solana_program::pubkey::Pubkey,
    /// SOL staker stake account (pda of `['stake::state::sol_staker_stake', stake state, config]`)
    pub sol_staker_stake: solana_program::pubkey::Pubkey,
    /// Validator stake account (pda of `['stake::state::validator_stake', validator, config]`)
    pub validator_stake: solana_program::pubkey::Pubkey,
    /// SOL stake account
    pub sol_stake: solana_program::pubkey::Pubkey,
    /// Destination account for withdrawn lamports
    pub destination: solana_program::pubkey::Pubkey,
    /// Stake history sysvar
    pub sysvar_stake_history: solana_program::pubkey::Pubkey,
    /// Paladin SOL Stake View program
    pub sol_stake_view_program: solana_program::pubkey::Pubkey,
}

impl HarvestSyncRewards {
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        self.instruction_with_remaining_accounts(&[])
    }
    #[allow(clippy::vec_init_then_push)]
    pub fn instruction_with_remaining_accounts(
        &self,
        remaining_accounts: &[solana_program::instruction::AccountMeta],
    ) -> solana_program::instruction::Instruction {
        let mut accounts = Vec::with_capacity(7 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.config,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.sol_staker_stake,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.validator_stake,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.sol_stake,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.destination,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.sysvar_stake_history,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.sol_stake_view_program,
            false,
        ));
        accounts.extend_from_slice(remaining_accounts);
        let data = HarvestSyncRewardsInstructionData::new()
            .try_to_vec()
            .unwrap();

        solana_program::instruction::Instruction {
            program_id: crate::PALADIN_STAKE_PROGRAM_ID,
            accounts,
            data,
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct HarvestSyncRewardsInstructionData {
    discriminator: u8,
}

impl HarvestSyncRewardsInstructionData {
    pub fn new() -> Self {
        Self { discriminator: 16 }
    }
}

impl Default for HarvestSyncRewardsInstructionData {
    fn default() -> Self {
        Self::new()
    }
}

/// Instruction builder for `HarvestSyncRewards`.
///
/// ### Accounts:
///
///   0. `[writable]` config
///   1. `[writable]` sol_staker_stake
///   2. `[writable]` validator_stake
///   3. `[]` sol_stake
///   4. `[writable]` destination
///   5. `[optional]` sysvar_stake_history (default to `SysvarStakeHistory1111111111111111111111111`)
///   6. `[]` sol_stake_view_program
#[derive(Clone, Debug, Default)]
pub struct HarvestSyncRewardsBuilder {
    config: Option<solana_program::pubkey::Pubkey>,
    sol_staker_stake: Option<solana_program::pubkey::Pubkey>,
    validator_stake: Option<solana_program::pubkey::Pubkey>,
    sol_stake: Option<solana_program::pubkey::Pubkey>,
    destination: Option<solana_program::pubkey::Pubkey>,
    sysvar_stake_history: Option<solana_program::pubkey::Pubkey>,
    sol_stake_view_program: Option<solana_program::pubkey::Pubkey>,
    __remaining_accounts: Vec<solana_program::instruction::AccountMeta>,
}

impl HarvestSyncRewardsBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    /// Stake config account
    #[inline(always)]
    pub fn config(&mut self, config: solana_program::pubkey::Pubkey) -> &mut Self {
        self.config = Some(config);
        self
    }
    /// SOL staker stake account (pda of `['stake::state::sol_staker_stake', stake state, config]`)
    #[inline(always)]
    pub fn sol_staker_stake(
        &mut self,
        sol_staker_stake: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.sol_staker_stake = Some(sol_staker_stake);
        self
    }
    /// Validator stake account (pda of `['stake::state::validator_stake', validator, config]`)
    #[inline(always)]
    pub fn validator_stake(
        &mut self,
        validator_stake: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.validator_stake = Some(validator_stake);
        self
    }
    /// SOL stake account
    #[inline(always)]
    pub fn sol_stake(&mut self, sol_stake: solana_program::pubkey::Pubkey) -> &mut Self {
        self.sol_stake = Some(sol_stake);
        self
    }
    /// Destination account for withdrawn lamports
    #[inline(always)]
    pub fn destination(&mut self, destination: solana_program::pubkey::Pubkey) -> &mut Self {
        self.destination = Some(destination);
        self
    }
    /// `[optional account, default to 'SysvarStakeHistory1111111111111111111111111']`
    /// Stake history sysvar
    #[inline(always)]
    pub fn sysvar_stake_history(
        &mut self,
        sysvar_stake_history: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.sysvar_stake_history = Some(sysvar_stake_history);
        self
    }
    /// Paladin SOL Stake View program
    #[inline(always)]
    pub fn sol_stake_view_program(
        &mut self,
        sol_stake_view_program: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.sol_stake_view_program = Some(sol_stake_view_program);
        self
    }
    /// Add an aditional account to the instruction.
    #[inline(always)]
    pub fn add_remaining_account(
        &mut self,
        account: solana_program::instruction::AccountMeta,
    ) -> &mut Self {
        self.__remaining_accounts.push(account);
        self
    }
    /// Add additional accounts to the instruction.
    #[inline(always)]
    pub fn add_remaining_accounts(
        &mut self,
        accounts: &[solana_program::instruction::AccountMeta],
    ) -> &mut Self {
        self.__remaining_accounts.extend_from_slice(accounts);
        self
    }
    #[allow(clippy::clone_on_copy)]
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        let accounts = HarvestSyncRewards {
            config: self.config.expect("config is not set"),
            sol_staker_stake: self.sol_staker_stake.expect("sol_staker_stake is not set"),
            validator_stake: self.validator_stake.expect("validator_stake is not set"),
            sol_stake: self.sol_stake.expect("sol_stake is not set"),
            destination: self.destination.expect("destination is not set"),
            sysvar_stake_history: self.sysvar_stake_history.unwrap_or(solana_program::pubkey!(
                "SysvarStakeHistory1111111111111111111111111"
            )),
            sol_stake_view_program: self
                .sol_stake_view_program
                .expect("sol_stake_view_program is not set"),
        };

        accounts.instruction_with_remaining_accounts(&self.__remaining_accounts)
    }
}

/// `harvest_sync_rewards` CPI accounts.
pub struct HarvestSyncRewardsCpiAccounts<'a, 'b> {
    /// Stake config account
    pub config: &'b solana_program::account_info::AccountInfo<'a>,
    /// SOL staker stake account (pda of `['stake::state::sol_staker_stake', stake state, config]`)
    pub sol_staker_stake: &'b solana_program::account_info::AccountInfo<'a>,
    /// Validator stake account (pda of `['stake::state::validator_stake', validator, config]`)
    pub validator_stake: &'b solana_program::account_info::AccountInfo<'a>,
    /// SOL stake account
    pub sol_stake: &'b solana_program::account_info::AccountInfo<'a>,
    /// Destination account for withdrawn lamports
    pub destination: &'b solana_program::account_info::AccountInfo<'a>,
    /// Stake history sysvar
    pub sysvar_stake_history: &'b solana_program::account_info::AccountInfo<'a>,
    /// Paladin SOL Stake View program
    pub sol_stake_view_program: &'b solana_program::account_info::AccountInfo<'a>,
}

/// `harvest_sync_rewards` CPI instruction.
pub struct HarvestSyncRewardsCpi<'a, 'b> {
    /// The program to invoke.
    pub __program: &'b solana_program::account_info::AccountInfo<'a>,
    /// Stake config account
    pub config: &'b solana_program::account_info::AccountInfo<'a>,
    /// SOL staker stake account (pda of `['stake::state::sol_staker_stake', stake state, config]`)
    pub sol_staker_stake: &'b solana_program::account_info::AccountInfo<'a>,
    /// Validator stake account (pda of `['stake::state::validator_stake', validator, config]`)
    pub validator_stake: &'b solana_program::account_info::AccountInfo<'a>,
    /// SOL stake account
    pub sol_stake: &'b solana_program::account_info::AccountInfo<'a>,
    /// Destination account for withdrawn lamports
    pub destination: &'b solana_program::account_info::AccountInfo<'a>,
    /// Stake history sysvar
    pub sysvar_stake_history: &'b solana_program::account_info::AccountInfo<'a>,
    /// Paladin SOL Stake View program
    pub sol_stake_view_program: &'b solana_program::account_info::AccountInfo<'a>,
}

impl<'a, 'b> HarvestSyncRewardsCpi<'a, 'b> {
    pub fn new(
        program: &'b solana_program::account_info::AccountInfo<'a>,
        accounts: HarvestSyncRewardsCpiAccounts<'a, 'b>,
    ) -> Self {
        Self {
            __program: program,
            config: accounts.config,
            sol_staker_stake: accounts.sol_staker_stake,
            validator_stake: accounts.validator_stake,
            sol_stake: accounts.sol_stake,
            destination: accounts.destination,
            sysvar_stake_history: accounts.sysvar_stake_history,
            sol_stake_view_program: accounts.sol_stake_view_program,
        }
    }
    #[inline(always)]
    pub fn invoke(&self) -> solana_program::entrypoint::ProgramResult {
        self.invoke_signed_with_remaining_accounts(&[], &[])
    }
    #[inline(always)]
    pub fn invoke_with_remaining_accounts(
        &self,
        remaining_accounts: &[(
            &'b solana_program::account_info::AccountInfo<'a>,
            bool,
            bool,
        )],
    ) -> solana_program::entrypoint::ProgramResult {
        self.invoke_signed_with_remaining_accounts(&[], remaining_accounts)
    }
    #[inline(always)]
    pub fn invoke_signed(
        &self,
        signers_seeds: &[&[&[u8]]],
    ) -> solana_program::entrypoint::ProgramResult {
        self.invoke_signed_with_remaining_accounts(signers_seeds, &[])
    }
    #[allow(clippy::clone_on_copy)]
    #[allow(clippy::vec_init_then_push)]
    pub fn invoke_signed_with_remaining_accounts(
        &self,
        signers_seeds: &[&[&[u8]]],
        remaining_accounts: &[(
            &'b solana_program::account_info::AccountInfo<'a>,
            bool,
            bool,
        )],
    ) -> solana_program::entrypoint::ProgramResult {
        let mut accounts = Vec::with_capacity(7 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.config.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.sol_staker_stake.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.validator_stake.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.sol_stake.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.destination.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.sysvar_stake_history.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.sol_stake_view_program.key,
            false,
        ));
        remaining_accounts.iter().for_each(|remaining_account| {
            accounts.push(solana_program::instruction::AccountMeta {
                pubkey: *remaining_account.0.key,
                is_signer: remaining_account.1,
                is_writable: remaining_account.2,
            })
        });
        let data = HarvestSyncRewardsInstructionData::new()
            .try_to_vec()
            .unwrap();

        let instruction = solana_program::instruction::Instruction {
            program_id: crate::PALADIN_STAKE_PROGRAM_ID,
            accounts,
            data,
        };
        let mut account_infos = Vec::with_capacity(7 + 1 + remaining_accounts.len());
        account_infos.push(self.__program.clone());
        account_infos.push(self.config.clone());
        account_infos.push(self.sol_staker_stake.clone());
        account_infos.push(self.validator_stake.clone());
        account_infos.push(self.sol_stake.clone());
        account_infos.push(self.destination.clone());
        account_infos.push(self.sysvar_stake_history.clone());
        account_infos.push(self.sol_stake_view_program.clone());
        remaining_accounts
            .iter()
            .for_each(|remaining_account| account_infos.push(remaining_account.0.clone()));

        if signers_seeds.is_empty() {
            solana_program::program::invoke(&instruction, &account_infos)
        } else {
            solana_program::program::invoke_signed(&instruction, &account_infos, signers_seeds)
        }
    }
}

/// Instruction builder for `HarvestSyncRewards` via CPI.
///
/// ### Accounts:
///
///   0. `[writable]` config
///   1. `[writable]` sol_staker_stake
///   2. `[writable]` validator_stake
///   3. `[]` sol_stake
///   4. `[writable]` destination
///   5. `[]` sysvar_stake_history
///   6. `[]` sol_stake_view_program
#[derive(Clone, Debug)]
pub struct HarvestSyncRewardsCpiBuilder<'a, 'b> {
    instruction: Box<HarvestSyncRewardsCpiBuilderInstruction<'a, 'b>>,
}

impl<'a, 'b> HarvestSyncRewardsCpiBuilder<'a, 'b> {
    pub fn new(program: &'b solana_program::account_info::AccountInfo<'a>) -> Self {
        let instruction = Box::new(HarvestSyncRewardsCpiBuilderInstruction {
            __program: program,
            config: None,
            sol_staker_stake: None,
            validator_stake: None,
            sol_stake: None,
            destination: None,
            sysvar_stake_history: None,
            sol_stake_view_program: None,
            __remaining_accounts: Vec::new(),
        });
        Self { instruction }
    }
    /// Stake config account
    #[inline(always)]
    pub fn config(
        &mut self,
        config: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.config = Some(config);
        self
    }
    /// SOL staker stake account (pda of `['stake::state::sol_staker_stake', stake state, config]`)
    #[inline(always)]
    pub fn sol_staker_stake(
        &mut self,
        sol_staker_stake: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.sol_staker_stake = Some(sol_staker_stake);
        self
    }
    /// Validator stake account (pda of `['stake::state::validator_stake', validator, config]`)
    #[inline(always)]
    pub fn validator_stake(
        &mut self,
        validator_stake: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.validator_stake = Some(validator_stake);
        self
    }
    /// SOL stake account
    #[inline(always)]
    pub fn sol_stake(
        &mut self,
        sol_stake: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.sol_stake = Some(sol_stake);
        self
    }
    /// Destination account for withdrawn lamports
    #[inline(always)]
    pub fn destination(
        &mut self,
        destination: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.destination = Some(destination);
        self
    }
    /// Stake history sysvar
    #[inline(always)]
    pub fn sysvar_stake_history(
        &mut self,
        sysvar_stake_history: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.sysvar_stake_history = Some(sysvar_stake_history);
        self
    }
    /// Paladin SOL Stake View program
    #[inline(always)]
    pub fn sol_stake_view_program(
        &mut self,
        sol_stake_view_program: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.sol_stake_view_program = Some(sol_stake_view_program);
        self
    }
    /// Add an additional account to the instruction.
    #[inline(always)]
    pub fn add_remaining_account(
        &mut self,
        account: &'b solana_program::account_info::AccountInfo<'a>,
        is_writable: bool,
        is_signer: bool,
    ) -> &mut Self {
        self.instruction
            .__remaining_accounts
            .push((account, is_writable, is_signer));
        self
    }
    /// Add additional accounts to the instruction.
    ///
    /// Each account is represented by a tuple of the `AccountInfo`, a `bool` indicating whether the account is writable or not,
    /// and a `bool` indicating whether the account is a signer or not.
    #[inline(always)]
    pub fn add_remaining_accounts(
        &mut self,
        accounts: &[(
            &'b solana_program::account_info::AccountInfo<'a>,
            bool,
            bool,
        )],
    ) -> &mut Self {
        self.instruction
            .__remaining_accounts
            .extend_from_slice(accounts);
        self
    }
    #[inline(always)]
    pub fn invoke(&self) -> solana_program::entrypoint::ProgramResult {
        self.invoke_signed(&[])
    }
    #[allow(clippy::clone_on_copy)]
    #[allow(clippy::vec_init_then_push)]
    pub fn invoke_signed(
        &self,
        signers_seeds: &[&[&[u8]]],
    ) -> solana_program::entrypoint::ProgramResult {
        let instruction = HarvestSyncRewardsCpi {
            __program: self.instruction.__program,

            config: self.instruction.config.expect("config is not set"),

            sol_staker_stake: self
                .instruction
                .sol_staker_stake
                .expect("sol_staker_stake is not set"),

            validator_stake: self
                .instruction
                .validator_stake
                .expect("validator_stake is not set"),

            sol_stake: self.instruction.sol_stake.expect("sol_stake is not set"),

            destination: self
                .instruction
                .destination
                .expect("destination is not set"),

            sysvar_stake_history: self
                .instruction
                .sysvar_stake_history
                .expect("sysvar_stake_history is not set"),

            sol_stake_view_program: self
                .instruction
                .sol_stake_view_program
                .expect("sol_stake_view_program is not set"),
        };
        instruction.invoke_signed_with_remaining_accounts(
            signers_seeds,
            &self.instruction.__remaining_accounts,
        )
    }
}

#[derive(Clone, Debug)]
struct HarvestSyncRewardsCpiBuilderInstruction<'a, 'b> {
    __program: &'b solana_program::account_info::AccountInfo<'a>,
    config: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    sol_staker_stake: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    validator_stake: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    sol_stake: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    destination: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    sysvar_stake_history: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    sol_stake_view_program: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    /// Additional instruction accounts `(AccountInfo, is_writable, is_signer)`.
    __remaining_accounts: Vec<(
        &'b solana_program::account_info::AccountInfo<'a>,
        bool,
        bool,
    )>,
}
