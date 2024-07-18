//! This code was AUTOGENERATED using the kinobi library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun kinobi to update it.
//!
//! <https://github.com/kinobi-so/kinobi>
//!

use crate::generated::types::ConfigField;
use borsh::BorshDeserialize;
use borsh::BorshSerialize;

/// Accounts.
pub struct UpdateConfig {
    /// Stake config account
    pub config: solana_program::pubkey::Pubkey,
    /// Stake config authority
    pub config_authority: solana_program::pubkey::Pubkey,
}

impl UpdateConfig {
    pub fn instruction(
        &self,
        args: UpdateConfigInstructionArgs,
    ) -> solana_program::instruction::Instruction {
        self.instruction_with_remaining_accounts(args, &[])
    }
    #[allow(clippy::vec_init_then_push)]
    pub fn instruction_with_remaining_accounts(
        &self,
        args: UpdateConfigInstructionArgs,
        remaining_accounts: &[solana_program::instruction::AccountMeta],
    ) -> solana_program::instruction::Instruction {
        let mut accounts = Vec::with_capacity(2 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.config,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.config_authority,
            true,
        ));
        accounts.extend_from_slice(remaining_accounts);
        let mut data = UpdateConfigInstructionData::new().try_to_vec().unwrap();
        let mut args = args.try_to_vec().unwrap();
        data.append(&mut args);

        solana_program::instruction::Instruction {
            program_id: crate::PALADIN_STAKE_PROGRAM_ID,
            accounts,
            data,
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct UpdateConfigInstructionData {
    discriminator: u8,
}

impl UpdateConfigInstructionData {
    pub fn new() -> Self {
        Self { discriminator: 10 }
    }
}

impl Default for UpdateConfigInstructionData {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UpdateConfigInstructionArgs {
    pub config_field: ConfigField,
}

/// Instruction builder for `UpdateConfig`.
///
/// ### Accounts:
///
///   0. `[writable]` config
///   1. `[signer]` config_authority
#[derive(Clone, Debug, Default)]
pub struct UpdateConfigBuilder {
    config: Option<solana_program::pubkey::Pubkey>,
    config_authority: Option<solana_program::pubkey::Pubkey>,
    config_field: Option<ConfigField>,
    __remaining_accounts: Vec<solana_program::instruction::AccountMeta>,
}

impl UpdateConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    /// Stake config account
    #[inline(always)]
    pub fn config(&mut self, config: solana_program::pubkey::Pubkey) -> &mut Self {
        self.config = Some(config);
        self
    }
    /// Stake config authority
    #[inline(always)]
    pub fn config_authority(
        &mut self,
        config_authority: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.config_authority = Some(config_authority);
        self
    }
    #[inline(always)]
    pub fn config_field(&mut self, config_field: ConfigField) -> &mut Self {
        self.config_field = Some(config_field);
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
        let accounts = UpdateConfig {
            config: self.config.expect("config is not set"),
            config_authority: self.config_authority.expect("config_authority is not set"),
        };
        let args = UpdateConfigInstructionArgs {
            config_field: self.config_field.clone().expect("config_field is not set"),
        };

        accounts.instruction_with_remaining_accounts(args, &self.__remaining_accounts)
    }
}

/// `update_config` CPI accounts.
pub struct UpdateConfigCpiAccounts<'a, 'b> {
    /// Stake config account
    pub config: &'b solana_program::account_info::AccountInfo<'a>,
    /// Stake config authority
    pub config_authority: &'b solana_program::account_info::AccountInfo<'a>,
}

/// `update_config` CPI instruction.
pub struct UpdateConfigCpi<'a, 'b> {
    /// The program to invoke.
    pub __program: &'b solana_program::account_info::AccountInfo<'a>,
    /// Stake config account
    pub config: &'b solana_program::account_info::AccountInfo<'a>,
    /// Stake config authority
    pub config_authority: &'b solana_program::account_info::AccountInfo<'a>,
    /// The arguments for the instruction.
    pub __args: UpdateConfigInstructionArgs,
}

impl<'a, 'b> UpdateConfigCpi<'a, 'b> {
    pub fn new(
        program: &'b solana_program::account_info::AccountInfo<'a>,
        accounts: UpdateConfigCpiAccounts<'a, 'b>,
        args: UpdateConfigInstructionArgs,
    ) -> Self {
        Self {
            __program: program,
            config: accounts.config,
            config_authority: accounts.config_authority,
            __args: args,
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
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.config_authority.key,
            true,
        ));
        remaining_accounts.iter().for_each(|remaining_account| {
            accounts.push(solana_program::instruction::AccountMeta {
                pubkey: *remaining_account.0.key,
                is_signer: remaining_account.1,
                is_writable: remaining_account.2,
            })
        });
        let mut data = UpdateConfigInstructionData::new().try_to_vec().unwrap();
        let mut args = self.__args.try_to_vec().unwrap();
        data.append(&mut args);

        let instruction = solana_program::instruction::Instruction {
            program_id: crate::PALADIN_STAKE_PROGRAM_ID,
            accounts,
            data,
        };
        let mut account_infos = Vec::with_capacity(2 + 1 + remaining_accounts.len());
        account_infos.push(self.__program.clone());
        account_infos.push(self.config.clone());
        account_infos.push(self.config_authority.clone());
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

/// Instruction builder for `UpdateConfig` via CPI.
///
/// ### Accounts:
///
///   0. `[writable]` config
///   1. `[signer]` config_authority
#[derive(Clone, Debug)]
pub struct UpdateConfigCpiBuilder<'a, 'b> {
    instruction: Box<UpdateConfigCpiBuilderInstruction<'a, 'b>>,
}

impl<'a, 'b> UpdateConfigCpiBuilder<'a, 'b> {
    pub fn new(program: &'b solana_program::account_info::AccountInfo<'a>) -> Self {
        let instruction = Box::new(UpdateConfigCpiBuilderInstruction {
            __program: program,
            config: None,
            config_authority: None,
            config_field: None,
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
    /// Stake config authority
    #[inline(always)]
    pub fn config_authority(
        &mut self,
        config_authority: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.config_authority = Some(config_authority);
        self
    }
    #[inline(always)]
    pub fn config_field(&mut self, config_field: ConfigField) -> &mut Self {
        self.instruction.config_field = Some(config_field);
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
        let args = UpdateConfigInstructionArgs {
            config_field: self
                .instruction
                .config_field
                .clone()
                .expect("config_field is not set"),
        };
        let instruction = UpdateConfigCpi {
            __program: self.instruction.__program,

            config: self.instruction.config.expect("config is not set"),

            config_authority: self
                .instruction
                .config_authority
                .expect("config_authority is not set"),
            __args: args,
        };
        instruction.invoke_signed_with_remaining_accounts(
            signers_seeds,
            &self.instruction.__remaining_accounts,
        )
    }
}

#[derive(Clone, Debug)]
struct UpdateConfigCpiBuilderInstruction<'a, 'b> {
    __program: &'b solana_program::account_info::AccountInfo<'a>,
    config: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    config_authority: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    config_field: Option<ConfigField>,
    /// Additional instruction accounts `(AccountInfo, is_writable, is_signer)`.
    __remaining_accounts: Vec<(
        &'b solana_program::account_info::AccountInfo<'a>,
        bool,
        bool,
    )>,
}
