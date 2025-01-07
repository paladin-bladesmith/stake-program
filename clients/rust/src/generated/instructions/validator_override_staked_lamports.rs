//! This code was AUTOGENERATED using the kinobi library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun kinobi to update it.
//!
//! <https://github.com/kinobi-so/kinobi>
//!

use borsh::BorshDeserialize;
use borsh::BorshSerialize;

/// Accounts.
pub struct ValidatorOverrideStakedLamports {
    /// Config
    pub config: solana_program::pubkey::Pubkey,
    /// Config authority
    pub config_authority: solana_program::pubkey::Pubkey,
    /// Validator stake
    pub validator_stake: solana_program::pubkey::Pubkey,
    /// Validator stake authority
    pub validator_stake_authority: solana_program::pubkey::Pubkey,
    /// Vault holder rewards
    pub vault_holder_rewards: solana_program::pubkey::Pubkey,
}

impl ValidatorOverrideStakedLamports {
    pub fn instruction(
        &self,
        args: ValidatorOverrideStakedLamportsInstructionArgs,
    ) -> solana_program::instruction::Instruction {
        self.instruction_with_remaining_accounts(args, &[])
    }
    #[allow(clippy::vec_init_then_push)]
    pub fn instruction_with_remaining_accounts(
        &self,
        args: ValidatorOverrideStakedLamportsInstructionArgs,
        remaining_accounts: &[solana_program::instruction::AccountMeta],
    ) -> solana_program::instruction::Instruction {
        let mut accounts = Vec::with_capacity(5 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.config,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.config_authority,
            true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.validator_stake,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.validator_stake_authority,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.vault_holder_rewards,
            false,
        ));
        accounts.extend_from_slice(remaining_accounts);
        let mut data = ValidatorOverrideStakedLamportsInstructionData::new()
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
pub struct ValidatorOverrideStakedLamportsInstructionData {
    discriminator: u8,
}

impl ValidatorOverrideStakedLamportsInstructionData {
    pub fn new() -> Self {
        Self { discriminator: 19 }
    }
}

impl Default for ValidatorOverrideStakedLamportsInstructionData {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ValidatorOverrideStakedLamportsInstructionArgs {
    pub amount_min: u64,
}

/// Instruction builder for `ValidatorOverrideStakedLamports`.
///
/// ### Accounts:
///
///   0. `[writable]` config
///   1. `[signer]` config_authority
///   2. `[writable]` validator_stake
///   3. `[writable]` validator_stake_authority
///   4. `[writable]` vault_holder_rewards
#[derive(Clone, Debug, Default)]
pub struct ValidatorOverrideStakedLamportsBuilder {
    config: Option<solana_program::pubkey::Pubkey>,
    config_authority: Option<solana_program::pubkey::Pubkey>,
    validator_stake: Option<solana_program::pubkey::Pubkey>,
    validator_stake_authority: Option<solana_program::pubkey::Pubkey>,
    vault_holder_rewards: Option<solana_program::pubkey::Pubkey>,
    amount_min: Option<u64>,
    __remaining_accounts: Vec<solana_program::instruction::AccountMeta>,
}

impl ValidatorOverrideStakedLamportsBuilder {
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
    /// Validator stake
    #[inline(always)]
    pub fn validator_stake(
        &mut self,
        validator_stake: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.validator_stake = Some(validator_stake);
        self
    }
    /// Validator stake authority
    #[inline(always)]
    pub fn validator_stake_authority(
        &mut self,
        validator_stake_authority: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.validator_stake_authority = Some(validator_stake_authority);
        self
    }
    /// Vault holder rewards
    #[inline(always)]
    pub fn vault_holder_rewards(
        &mut self,
        vault_holder_rewards: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.vault_holder_rewards = Some(vault_holder_rewards);
        self
    }
    #[inline(always)]
    pub fn amount_min(&mut self, amount_min: u64) -> &mut Self {
        self.amount_min = Some(amount_min);
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
        let accounts = ValidatorOverrideStakedLamports {
            config: self.config.expect("config is not set"),
            config_authority: self.config_authority.expect("config_authority is not set"),
            validator_stake: self.validator_stake.expect("validator_stake is not set"),
            validator_stake_authority: self
                .validator_stake_authority
                .expect("validator_stake_authority is not set"),
            vault_holder_rewards: self
                .vault_holder_rewards
                .expect("vault_holder_rewards is not set"),
        };
        let args = ValidatorOverrideStakedLamportsInstructionArgs {
            amount_min: self.amount_min.clone().expect("amount_min is not set"),
        };

        accounts.instruction_with_remaining_accounts(args, &self.__remaining_accounts)
    }
}

/// `validator_override_staked_lamports` CPI accounts.
pub struct ValidatorOverrideStakedLamportsCpiAccounts<'a, 'b> {
    /// Config
    pub config: &'b solana_program::account_info::AccountInfo<'a>,
    /// Config authority
    pub config_authority: &'b solana_program::account_info::AccountInfo<'a>,
    /// Validator stake
    pub validator_stake: &'b solana_program::account_info::AccountInfo<'a>,
    /// Validator stake authority
    pub validator_stake_authority: &'b solana_program::account_info::AccountInfo<'a>,
    /// Vault holder rewards
    pub vault_holder_rewards: &'b solana_program::account_info::AccountInfo<'a>,
}

/// `validator_override_staked_lamports` CPI instruction.
pub struct ValidatorOverrideStakedLamportsCpi<'a, 'b> {
    /// The program to invoke.
    pub __program: &'b solana_program::account_info::AccountInfo<'a>,
    /// Config
    pub config: &'b solana_program::account_info::AccountInfo<'a>,
    /// Config authority
    pub config_authority: &'b solana_program::account_info::AccountInfo<'a>,
    /// Validator stake
    pub validator_stake: &'b solana_program::account_info::AccountInfo<'a>,
    /// Validator stake authority
    pub validator_stake_authority: &'b solana_program::account_info::AccountInfo<'a>,
    /// Vault holder rewards
    pub vault_holder_rewards: &'b solana_program::account_info::AccountInfo<'a>,
    /// The arguments for the instruction.
    pub __args: ValidatorOverrideStakedLamportsInstructionArgs,
}

impl<'a, 'b> ValidatorOverrideStakedLamportsCpi<'a, 'b> {
    pub fn new(
        program: &'b solana_program::account_info::AccountInfo<'a>,
        accounts: ValidatorOverrideStakedLamportsCpiAccounts<'a, 'b>,
        args: ValidatorOverrideStakedLamportsInstructionArgs,
    ) -> Self {
        Self {
            __program: program,
            config: accounts.config,
            config_authority: accounts.config_authority,
            validator_stake: accounts.validator_stake,
            validator_stake_authority: accounts.validator_stake_authority,
            vault_holder_rewards: accounts.vault_holder_rewards,
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
        let mut accounts = Vec::with_capacity(5 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.config.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.config_authority.key,
            true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.validator_stake.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.validator_stake_authority.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.vault_holder_rewards.key,
            false,
        ));
        remaining_accounts.iter().for_each(|remaining_account| {
            accounts.push(solana_program::instruction::AccountMeta {
                pubkey: *remaining_account.0.key,
                is_signer: remaining_account.1,
                is_writable: remaining_account.2,
            })
        });
        let mut data = ValidatorOverrideStakedLamportsInstructionData::new()
            .try_to_vec()
            .unwrap();
        let mut args = self.__args.try_to_vec().unwrap();
        data.append(&mut args);

        let instruction = solana_program::instruction::Instruction {
            program_id: crate::PALADIN_STAKE_PROGRAM_ID,
            accounts,
            data,
        };
        let mut account_infos = Vec::with_capacity(5 + 1 + remaining_accounts.len());
        account_infos.push(self.__program.clone());
        account_infos.push(self.config.clone());
        account_infos.push(self.config_authority.clone());
        account_infos.push(self.validator_stake.clone());
        account_infos.push(self.validator_stake_authority.clone());
        account_infos.push(self.vault_holder_rewards.clone());
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

/// Instruction builder for `ValidatorOverrideStakedLamports` via CPI.
///
/// ### Accounts:
///
///   0. `[writable]` config
///   1. `[signer]` config_authority
///   2. `[writable]` validator_stake
///   3. `[writable]` validator_stake_authority
///   4. `[writable]` vault_holder_rewards
#[derive(Clone, Debug)]
pub struct ValidatorOverrideStakedLamportsCpiBuilder<'a, 'b> {
    instruction: Box<ValidatorOverrideStakedLamportsCpiBuilderInstruction<'a, 'b>>,
}

impl<'a, 'b> ValidatorOverrideStakedLamportsCpiBuilder<'a, 'b> {
    pub fn new(program: &'b solana_program::account_info::AccountInfo<'a>) -> Self {
        let instruction = Box::new(ValidatorOverrideStakedLamportsCpiBuilderInstruction {
            __program: program,
            config: None,
            config_authority: None,
            validator_stake: None,
            validator_stake_authority: None,
            vault_holder_rewards: None,
            amount_min: None,
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
    /// Validator stake
    #[inline(always)]
    pub fn validator_stake(
        &mut self,
        validator_stake: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.validator_stake = Some(validator_stake);
        self
    }
    /// Validator stake authority
    #[inline(always)]
    pub fn validator_stake_authority(
        &mut self,
        validator_stake_authority: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.validator_stake_authority = Some(validator_stake_authority);
        self
    }
    /// Vault holder rewards
    #[inline(always)]
    pub fn vault_holder_rewards(
        &mut self,
        vault_holder_rewards: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.vault_holder_rewards = Some(vault_holder_rewards);
        self
    }
    #[inline(always)]
    pub fn amount_min(&mut self, amount_min: u64) -> &mut Self {
        self.instruction.amount_min = Some(amount_min);
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
        let args = ValidatorOverrideStakedLamportsInstructionArgs {
            amount_min: self
                .instruction
                .amount_min
                .clone()
                .expect("amount_min is not set"),
        };
        let instruction = ValidatorOverrideStakedLamportsCpi {
            __program: self.instruction.__program,

            config: self.instruction.config.expect("config is not set"),

            config_authority: self
                .instruction
                .config_authority
                .expect("config_authority is not set"),

            validator_stake: self
                .instruction
                .validator_stake
                .expect("validator_stake is not set"),

            validator_stake_authority: self
                .instruction
                .validator_stake_authority
                .expect("validator_stake_authority is not set"),

            vault_holder_rewards: self
                .instruction
                .vault_holder_rewards
                .expect("vault_holder_rewards is not set"),
            __args: args,
        };
        instruction.invoke_signed_with_remaining_accounts(
            signers_seeds,
            &self.instruction.__remaining_accounts,
        )
    }
}

#[derive(Clone, Debug)]
struct ValidatorOverrideStakedLamportsCpiBuilderInstruction<'a, 'b> {
    __program: &'b solana_program::account_info::AccountInfo<'a>,
    config: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    config_authority: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    validator_stake: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    validator_stake_authority: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    vault_holder_rewards: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    amount_min: Option<u64>,
    /// Additional instruction accounts `(AccountInfo, is_writable, is_signer)`.
    __remaining_accounts: Vec<(
        &'b solana_program::account_info::AccountInfo<'a>,
        bool,
        bool,
    )>,
}
