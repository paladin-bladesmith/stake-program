//! This code was AUTOGENERATED using the kinobi library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun kinobi to update it.
//!
//! <https://github.com/kinobi-so/kinobi>
//!

use borsh::{BorshDeserialize, BorshSerialize};

/// Accounts.
pub struct InactivateStake {
    /// Stake config account
    pub config: solana_program::pubkey::Pubkey,
    /// Validator stake account (pda of `['stake::state::stake', validator, config]`)
    pub stake: solana_program::pubkey::Pubkey,
}

impl InactivateStake {
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        self.instruction_with_remaining_accounts(&[])
    }
    #[allow(clippy::vec_init_then_push)]
    pub fn instruction_with_remaining_accounts(
        &self,
        remaining_accounts: &[solana_program::instruction::AccountMeta],
    ) -> solana_program::instruction::Instruction {
        let mut accounts = Vec::with_capacity(2 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.config,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.stake, false,
        ));
        accounts.extend_from_slice(remaining_accounts);
        let data = InactivateStakeInstructionData::new().try_to_vec().unwrap();

        solana_program::instruction::Instruction {
            program_id: crate::STAKE_ID,
            accounts,
            data,
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct InactivateStakeInstructionData {
    discriminator: u8,
}

impl InactivateStakeInstructionData {
    pub fn new() -> Self {
        Self { discriminator: 4 }
    }
}

impl Default for InactivateStakeInstructionData {
    fn default() -> Self {
        Self::new()
    }
}

/// Instruction builder for `InactivateStake`.
///
/// ### Accounts:
///
///   0. `[writable]` config
///   1. `[writable]` stake
#[derive(Clone, Debug, Default)]
pub struct InactivateStakeBuilder {
    config: Option<solana_program::pubkey::Pubkey>,
    stake: Option<solana_program::pubkey::Pubkey>,
    __remaining_accounts: Vec<solana_program::instruction::AccountMeta>,
}

impl InactivateStakeBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    /// Stake config account
    #[inline(always)]
    pub fn config(&mut self, config: solana_program::pubkey::Pubkey) -> &mut Self {
        self.config = Some(config);
        self
    }
    /// Validator stake account (pda of `['stake::state::stake', validator, config]`)
    #[inline(always)]
    pub fn stake(&mut self, stake: solana_program::pubkey::Pubkey) -> &mut Self {
        self.stake = Some(stake);
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
        let accounts = InactivateStake {
            config: self.config.expect("config is not set"),
            stake: self.stake.expect("stake is not set"),
        };

        accounts.instruction_with_remaining_accounts(&self.__remaining_accounts)
    }
}

/// `inactivate_stake` CPI accounts.
pub struct InactivateStakeCpiAccounts<'a, 'b> {
    /// Stake config account
    pub config: &'b solana_program::account_info::AccountInfo<'a>,
    /// Validator stake account (pda of `['stake::state::stake', validator, config]`)
    pub stake: &'b solana_program::account_info::AccountInfo<'a>,
}

/// `inactivate_stake` CPI instruction.
pub struct InactivateStakeCpi<'a, 'b> {
    /// The program to invoke.
    pub __program: &'b solana_program::account_info::AccountInfo<'a>,
    /// Stake config account
    pub config: &'b solana_program::account_info::AccountInfo<'a>,
    /// Validator stake account (pda of `['stake::state::stake', validator, config]`)
    pub stake: &'b solana_program::account_info::AccountInfo<'a>,
}

impl<'a, 'b> InactivateStakeCpi<'a, 'b> {
    pub fn new(
        program: &'b solana_program::account_info::AccountInfo<'a>,
        accounts: InactivateStakeCpiAccounts<'a, 'b>,
    ) -> Self {
        Self {
            __program: program,
            config: accounts.config,
            stake: accounts.stake,
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
        let mut accounts = Vec::with_capacity(2 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.config.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.stake.key,
            false,
        ));
        remaining_accounts.iter().for_each(|remaining_account| {
            accounts.push(solana_program::instruction::AccountMeta {
                pubkey: *remaining_account.0.key,
                is_signer: remaining_account.1,
                is_writable: remaining_account.2,
            })
        });
        let data = InactivateStakeInstructionData::new().try_to_vec().unwrap();

        let instruction = solana_program::instruction::Instruction {
            program_id: crate::STAKE_ID,
            accounts,
            data,
        };
        let mut account_infos = Vec::with_capacity(2 + 1 + remaining_accounts.len());
        account_infos.push(self.__program.clone());
        account_infos.push(self.config.clone());
        account_infos.push(self.stake.clone());
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

/// Instruction builder for `InactivateStake` via CPI.
///
/// ### Accounts:
///
///   0. `[writable]` config
///   1. `[writable]` stake
#[derive(Clone, Debug)]
pub struct InactivateStakeCpiBuilder<'a, 'b> {
    instruction: Box<InactivateStakeCpiBuilderInstruction<'a, 'b>>,
}

impl<'a, 'b> InactivateStakeCpiBuilder<'a, 'b> {
    pub fn new(program: &'b solana_program::account_info::AccountInfo<'a>) -> Self {
        let instruction = Box::new(InactivateStakeCpiBuilderInstruction {
            __program: program,
            config: None,
            stake: None,
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
    /// Validator stake account (pda of `['stake::state::stake', validator, config]`)
    #[inline(always)]
    pub fn stake(&mut self, stake: &'b solana_program::account_info::AccountInfo<'a>) -> &mut Self {
        self.instruction.stake = Some(stake);
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
        let instruction = InactivateStakeCpi {
            __program: self.instruction.__program,

            config: self.instruction.config.expect("config is not set"),

            stake: self.instruction.stake.expect("stake is not set"),
        };
        instruction.invoke_signed_with_remaining_accounts(
            signers_seeds,
            &self.instruction.__remaining_accounts,
        )
    }
}

#[derive(Clone, Debug)]
struct InactivateStakeCpiBuilderInstruction<'a, 'b> {
    __program: &'b solana_program::account_info::AccountInfo<'a>,
    config: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    stake: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    /// Additional instruction accounts `(AccountInfo, is_writable, is_signer)`.
    __remaining_accounts: Vec<(
        &'b solana_program::account_info::AccountInfo<'a>,
        bool,
        bool,
    )>,
}
