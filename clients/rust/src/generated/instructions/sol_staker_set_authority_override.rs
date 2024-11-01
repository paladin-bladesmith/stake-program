//! This code was AUTOGENERATED using the kinobi library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun kinobi to update it.
//!
//! <https://github.com/kinobi-so/kinobi>
//!

use borsh::BorshDeserialize;
use borsh::BorshSerialize;
use solana_program::pubkey::Pubkey;

/// Accounts.
pub struct SolStakerSetAuthorityOverride {
    /// Config
    pub config: solana_program::pubkey::Pubkey,
    /// Config authority
    pub config_authority: solana_program::pubkey::Pubkey,
    /// Sol staker authority override
    pub sol_staker_authority_override: solana_program::pubkey::Pubkey,
    /// System program
    pub system_program: Option<solana_program::pubkey::Pubkey>,
}

impl SolStakerSetAuthorityOverride {
    pub fn instruction(
        &self,
        args: SolStakerSetAuthorityOverrideInstructionArgs,
    ) -> solana_program::instruction::Instruction {
        self.instruction_with_remaining_accounts(args, &[])
    }
    #[allow(clippy::vec_init_then_push)]
    pub fn instruction_with_remaining_accounts(
        &self,
        args: SolStakerSetAuthorityOverrideInstructionArgs,
        remaining_accounts: &[solana_program::instruction::AccountMeta],
    ) -> solana_program::instruction::Instruction {
        let mut accounts = Vec::with_capacity(4 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.config,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.config_authority,
            true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.sol_staker_authority_override,
            false,
        ));
        if let Some(system_program) = self.system_program {
            accounts.push(solana_program::instruction::AccountMeta::new_readonly(
                system_program,
                false,
            ));
        } else {
            accounts.push(solana_program::instruction::AccountMeta::new_readonly(
                crate::PALADIN_STAKE_PROGRAM_ID,
                false,
            ));
        }
        accounts.extend_from_slice(remaining_accounts);
        let mut data = SolStakerSetAuthorityOverrideInstructionData::new()
            .try_to_vec()
            .unwrap();
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
pub struct SolStakerSetAuthorityOverrideInstructionData {
    discriminator: u8,
}

impl SolStakerSetAuthorityOverrideInstructionData {
    pub fn new() -> Self {
        Self { discriminator: 18 }
    }
}

impl Default for SolStakerSetAuthorityOverrideInstructionData {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SolStakerSetAuthorityOverrideInstructionArgs {
    pub authority_original: Pubkey,
    pub authority_override: Pubkey,
}

/// Instruction builder for `SolStakerSetAuthorityOverride`.
///
/// ### Accounts:
///
///   0. `[]` config
///   1. `[signer]` config_authority
///   2. `[writable]` sol_staker_authority_override
///   3. `[optional]` system_program
#[derive(Clone, Debug, Default)]
pub struct SolStakerSetAuthorityOverrideBuilder {
    config: Option<solana_program::pubkey::Pubkey>,
    config_authority: Option<solana_program::pubkey::Pubkey>,
    sol_staker_authority_override: Option<solana_program::pubkey::Pubkey>,
    system_program: Option<solana_program::pubkey::Pubkey>,
    authority_original: Option<Pubkey>,
    authority_override: Option<Pubkey>,
    __remaining_accounts: Vec<solana_program::instruction::AccountMeta>,
}

impl SolStakerSetAuthorityOverrideBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    /// Config
    #[inline(always)]
    pub fn config(&mut self, config: solana_program::pubkey::Pubkey) -> &mut Self {
        self.config = Some(config);
        self
    }
    /// Config authority
    #[inline(always)]
    pub fn config_authority(
        &mut self,
        config_authority: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.config_authority = Some(config_authority);
        self
    }
    /// Sol staker authority override
    #[inline(always)]
    pub fn sol_staker_authority_override(
        &mut self,
        sol_staker_authority_override: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.sol_staker_authority_override = Some(sol_staker_authority_override);
        self
    }
    /// `[optional account]`
    /// System program
    #[inline(always)]
    pub fn system_program(
        &mut self,
        system_program: Option<solana_program::pubkey::Pubkey>,
    ) -> &mut Self {
        self.system_program = system_program;
        self
    }
    #[inline(always)]
    pub fn authority_original(&mut self, authority_original: Pubkey) -> &mut Self {
        self.authority_original = Some(authority_original);
        self
    }
    #[inline(always)]
    pub fn authority_override(&mut self, authority_override: Pubkey) -> &mut Self {
        self.authority_override = Some(authority_override);
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
        let accounts = SolStakerSetAuthorityOverride {
            config: self.config.expect("config is not set"),
            config_authority: self.config_authority.expect("config_authority is not set"),
            sol_staker_authority_override: self
                .sol_staker_authority_override
                .expect("sol_staker_authority_override is not set"),
            system_program: self.system_program,
        };
        let args = SolStakerSetAuthorityOverrideInstructionArgs {
            authority_original: self
                .authority_original
                .clone()
                .expect("authority_original is not set"),
            authority_override: self
                .authority_override
                .clone()
                .expect("authority_override is not set"),
        };

        accounts.instruction_with_remaining_accounts(args, &self.__remaining_accounts)
    }
}

/// `sol_staker_set_authority_override` CPI accounts.
pub struct SolStakerSetAuthorityOverrideCpiAccounts<'a, 'b> {
    /// Config
    pub config: &'b solana_program::account_info::AccountInfo<'a>,
    /// Config authority
    pub config_authority: &'b solana_program::account_info::AccountInfo<'a>,
    /// Sol staker authority override
    pub sol_staker_authority_override: &'b solana_program::account_info::AccountInfo<'a>,
    /// System program
    pub system_program: Option<&'b solana_program::account_info::AccountInfo<'a>>,
}

/// `sol_staker_set_authority_override` CPI instruction.
pub struct SolStakerSetAuthorityOverrideCpi<'a, 'b> {
    /// The program to invoke.
    pub __program: &'b solana_program::account_info::AccountInfo<'a>,
    /// Config
    pub config: &'b solana_program::account_info::AccountInfo<'a>,
    /// Config authority
    pub config_authority: &'b solana_program::account_info::AccountInfo<'a>,
    /// Sol staker authority override
    pub sol_staker_authority_override: &'b solana_program::account_info::AccountInfo<'a>,
    /// System program
    pub system_program: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    /// The arguments for the instruction.
    pub __args: SolStakerSetAuthorityOverrideInstructionArgs,
}

impl<'a, 'b> SolStakerSetAuthorityOverrideCpi<'a, 'b> {
    pub fn new(
        program: &'b solana_program::account_info::AccountInfo<'a>,
        accounts: SolStakerSetAuthorityOverrideCpiAccounts<'a, 'b>,
        args: SolStakerSetAuthorityOverrideInstructionArgs,
    ) -> Self {
        Self {
            __program: program,
            config: accounts.config,
            config_authority: accounts.config_authority,
            sol_staker_authority_override: accounts.sol_staker_authority_override,
            system_program: accounts.system_program,
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
        let mut accounts = Vec::with_capacity(4 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.config.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.config_authority.key,
            true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.sol_staker_authority_override.key,
            false,
        ));
        if let Some(system_program) = self.system_program {
            accounts.push(solana_program::instruction::AccountMeta::new_readonly(
                *system_program.key,
                false,
            ));
        } else {
            accounts.push(solana_program::instruction::AccountMeta::new_readonly(
                crate::PALADIN_STAKE_PROGRAM_ID,
                false,
            ));
        }
        remaining_accounts.iter().for_each(|remaining_account| {
            accounts.push(solana_program::instruction::AccountMeta {
                pubkey: *remaining_account.0.key,
                is_signer: remaining_account.1,
                is_writable: remaining_account.2,
            })
        });
        let mut data = SolStakerSetAuthorityOverrideInstructionData::new()
            .try_to_vec()
            .unwrap();
        let mut args = self.__args.try_to_vec().unwrap();
        data.append(&mut args);

        let instruction = solana_program::instruction::Instruction {
            program_id: crate::PALADIN_STAKE_PROGRAM_ID,
            accounts,
            data,
        };
        let mut account_infos = Vec::with_capacity(4 + 1 + remaining_accounts.len());
        account_infos.push(self.__program.clone());
        account_infos.push(self.config.clone());
        account_infos.push(self.config_authority.clone());
        account_infos.push(self.sol_staker_authority_override.clone());
        if let Some(system_program) = self.system_program {
            account_infos.push(system_program.clone());
        }
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

/// Instruction builder for `SolStakerSetAuthorityOverride` via CPI.
///
/// ### Accounts:
///
///   0. `[]` config
///   1. `[signer]` config_authority
///   2. `[writable]` sol_staker_authority_override
///   3. `[optional]` system_program
#[derive(Clone, Debug)]
pub struct SolStakerSetAuthorityOverrideCpiBuilder<'a, 'b> {
    instruction: Box<SolStakerSetAuthorityOverrideCpiBuilderInstruction<'a, 'b>>,
}

impl<'a, 'b> SolStakerSetAuthorityOverrideCpiBuilder<'a, 'b> {
    pub fn new(program: &'b solana_program::account_info::AccountInfo<'a>) -> Self {
        let instruction = Box::new(SolStakerSetAuthorityOverrideCpiBuilderInstruction {
            __program: program,
            config: None,
            config_authority: None,
            sol_staker_authority_override: None,
            system_program: None,
            authority_original: None,
            authority_override: None,
            __remaining_accounts: Vec::new(),
        });
        Self { instruction }
    }
    /// Config
    #[inline(always)]
    pub fn config(
        &mut self,
        config: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.config = Some(config);
        self
    }
    /// Config authority
    #[inline(always)]
    pub fn config_authority(
        &mut self,
        config_authority: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.config_authority = Some(config_authority);
        self
    }
    /// Sol staker authority override
    #[inline(always)]
    pub fn sol_staker_authority_override(
        &mut self,
        sol_staker_authority_override: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.sol_staker_authority_override = Some(sol_staker_authority_override);
        self
    }
    /// `[optional account]`
    /// System program
    #[inline(always)]
    pub fn system_program(
        &mut self,
        system_program: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    ) -> &mut Self {
        self.instruction.system_program = system_program;
        self
    }
    #[inline(always)]
    pub fn authority_original(&mut self, authority_original: Pubkey) -> &mut Self {
        self.instruction.authority_original = Some(authority_original);
        self
    }
    #[inline(always)]
    pub fn authority_override(&mut self, authority_override: Pubkey) -> &mut Self {
        self.instruction.authority_override = Some(authority_override);
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
        let args = SolStakerSetAuthorityOverrideInstructionArgs {
            authority_original: self
                .instruction
                .authority_original
                .clone()
                .expect("authority_original is not set"),
            authority_override: self
                .instruction
                .authority_override
                .clone()
                .expect("authority_override is not set"),
        };
        let instruction = SolStakerSetAuthorityOverrideCpi {
            __program: self.instruction.__program,

            config: self.instruction.config.expect("config is not set"),

            config_authority: self
                .instruction
                .config_authority
                .expect("config_authority is not set"),

            sol_staker_authority_override: self
                .instruction
                .sol_staker_authority_override
                .expect("sol_staker_authority_override is not set"),

            system_program: self.instruction.system_program,
            __args: args,
        };
        instruction.invoke_signed_with_remaining_accounts(
            signers_seeds,
            &self.instruction.__remaining_accounts,
        )
    }
}

#[derive(Clone, Debug)]
struct SolStakerSetAuthorityOverrideCpiBuilderInstruction<'a, 'b> {
    __program: &'b solana_program::account_info::AccountInfo<'a>,
    config: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    config_authority: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    sol_staker_authority_override: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    system_program: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    authority_original: Option<Pubkey>,
    authority_override: Option<Pubkey>,
    /// Additional instruction accounts `(AccountInfo, is_writable, is_signer)`.
    __remaining_accounts: Vec<(
        &'b solana_program::account_info::AccountInfo<'a>,
        bool,
        bool,
    )>,
}