//! This code was AUTOGENERATED using the kinobi library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun kinobi to update it.
//!
//! <https://github.com/kinobi-so/kinobi>
//!

use borsh::BorshDeserialize;
use borsh::BorshSerialize;

/// Accounts.
pub struct SolStakerUpdateAuthority {
    /// Config
    pub config: solana_program::pubkey::Pubkey,
    /// Sol staker stake
    pub sol_staker_stake: solana_program::pubkey::Pubkey,
    /// Sol staker authority override
    pub sol_staker_authority_override: solana_program::pubkey::Pubkey,
}

impl SolStakerUpdateAuthority {
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        self.instruction_with_remaining_accounts(&[])
    }
    #[allow(clippy::vec_init_then_push)]
    pub fn instruction_with_remaining_accounts(
        &self,
        remaining_accounts: &[solana_program::instruction::AccountMeta],
    ) -> solana_program::instruction::Instruction {
        let mut accounts = Vec::with_capacity(3 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.config,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.sol_staker_stake,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.sol_staker_authority_override,
            false,
        ));
        accounts.extend_from_slice(remaining_accounts);
        let data = SolStakerUpdateAuthorityInstructionData::new()
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
pub struct SolStakerUpdateAuthorityInstructionData {
    discriminator: u8,
}

impl SolStakerUpdateAuthorityInstructionData {
    pub fn new() -> Self {
        Self { discriminator: 17 }
    }
}

impl Default for SolStakerUpdateAuthorityInstructionData {
    fn default() -> Self {
        Self::new()
    }
}

/// Instruction builder for `SolStakerUpdateAuthority`.
///
/// ### Accounts:
///
///   0. `[]` config
///   1. `[writable]` sol_staker_stake
///   2. `[]` sol_staker_authority_override
#[derive(Clone, Debug, Default)]
pub struct SolStakerUpdateAuthorityBuilder {
    config: Option<solana_program::pubkey::Pubkey>,
    sol_staker_stake: Option<solana_program::pubkey::Pubkey>,
    sol_staker_authority_override: Option<solana_program::pubkey::Pubkey>,
    __remaining_accounts: Vec<solana_program::instruction::AccountMeta>,
}

impl SolStakerUpdateAuthorityBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    /// Config
    #[inline(always)]
    pub fn config(&mut self, config: solana_program::pubkey::Pubkey) -> &mut Self {
        self.config = Some(config);
        self
    }
    /// Sol staker stake
    #[inline(always)]
    pub fn sol_staker_stake(
        &mut self,
        sol_staker_stake: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.sol_staker_stake = Some(sol_staker_stake);
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
        let accounts = SolStakerUpdateAuthority {
            config: self.config.expect("config is not set"),
            sol_staker_stake: self.sol_staker_stake.expect("sol_staker_stake is not set"),
            sol_staker_authority_override: self
                .sol_staker_authority_override
                .expect("sol_staker_authority_override is not set"),
        };

        accounts.instruction_with_remaining_accounts(&self.__remaining_accounts)
    }
}

/// `sol_staker_update_authority` CPI accounts.
pub struct SolStakerUpdateAuthorityCpiAccounts<'a, 'b> {
    /// Config
    pub config: &'b solana_program::account_info::AccountInfo<'a>,
    /// Sol staker stake
    pub sol_staker_stake: &'b solana_program::account_info::AccountInfo<'a>,
    /// Sol staker authority override
    pub sol_staker_authority_override: &'b solana_program::account_info::AccountInfo<'a>,
}

/// `sol_staker_update_authority` CPI instruction.
pub struct SolStakerUpdateAuthorityCpi<'a, 'b> {
    /// The program to invoke.
    pub __program: &'b solana_program::account_info::AccountInfo<'a>,
    /// Config
    pub config: &'b solana_program::account_info::AccountInfo<'a>,
    /// Sol staker stake
    pub sol_staker_stake: &'b solana_program::account_info::AccountInfo<'a>,
    /// Sol staker authority override
    pub sol_staker_authority_override: &'b solana_program::account_info::AccountInfo<'a>,
}

impl<'a, 'b> SolStakerUpdateAuthorityCpi<'a, 'b> {
    pub fn new(
        program: &'b solana_program::account_info::AccountInfo<'a>,
        accounts: SolStakerUpdateAuthorityCpiAccounts<'a, 'b>,
    ) -> Self {
        Self {
            __program: program,
            config: accounts.config,
            sol_staker_stake: accounts.sol_staker_stake,
            sol_staker_authority_override: accounts.sol_staker_authority_override,
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
        let mut accounts = Vec::with_capacity(3 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.config.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.sol_staker_stake.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.sol_staker_authority_override.key,
            false,
        ));
        remaining_accounts.iter().for_each(|remaining_account| {
            accounts.push(solana_program::instruction::AccountMeta {
                pubkey: *remaining_account.0.key,
                is_signer: remaining_account.1,
                is_writable: remaining_account.2,
            })
        });
        let data = SolStakerUpdateAuthorityInstructionData::new()
            .try_to_vec()
            .unwrap();

        let instruction = solana_program::instruction::Instruction {
            program_id: crate::PALADIN_STAKE_PROGRAM_ID,
            accounts,
            data,
        };
        let mut account_infos = Vec::with_capacity(3 + 1 + remaining_accounts.len());
        account_infos.push(self.__program.clone());
        account_infos.push(self.config.clone());
        account_infos.push(self.sol_staker_stake.clone());
        account_infos.push(self.sol_staker_authority_override.clone());
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

/// Instruction builder for `SolStakerUpdateAuthority` via CPI.
///
/// ### Accounts:
///
///   0. `[]` config
///   1. `[writable]` sol_staker_stake
///   2. `[]` sol_staker_authority_override
#[derive(Clone, Debug)]
pub struct SolStakerUpdateAuthorityCpiBuilder<'a, 'b> {
    instruction: Box<SolStakerUpdateAuthorityCpiBuilderInstruction<'a, 'b>>,
}

impl<'a, 'b> SolStakerUpdateAuthorityCpiBuilder<'a, 'b> {
    pub fn new(program: &'b solana_program::account_info::AccountInfo<'a>) -> Self {
        let instruction = Box::new(SolStakerUpdateAuthorityCpiBuilderInstruction {
            __program: program,
            config: None,
            sol_staker_stake: None,
            sol_staker_authority_override: None,
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
    /// Sol staker stake
    #[inline(always)]
    pub fn sol_staker_stake(
        &mut self,
        sol_staker_stake: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.sol_staker_stake = Some(sol_staker_stake);
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
        let instruction = SolStakerUpdateAuthorityCpi {
            __program: self.instruction.__program,

            config: self.instruction.config.expect("config is not set"),

            sol_staker_stake: self
                .instruction
                .sol_staker_stake
                .expect("sol_staker_stake is not set"),

            sol_staker_authority_override: self
                .instruction
                .sol_staker_authority_override
                .expect("sol_staker_authority_override is not set"),
        };
        instruction.invoke_signed_with_remaining_accounts(
            signers_seeds,
            &self.instruction.__remaining_accounts,
        )
    }
}

#[derive(Clone, Debug)]
struct SolStakerUpdateAuthorityCpiBuilderInstruction<'a, 'b> {
    __program: &'b solana_program::account_info::AccountInfo<'a>,
    config: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    sol_staker_stake: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    sol_staker_authority_override: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    /// Additional instruction accounts `(AccountInfo, is_writable, is_signer)`.
    __remaining_accounts: Vec<(
        &'b solana_program::account_info::AccountInfo<'a>,
        bool,
        bool,
    )>,
}
