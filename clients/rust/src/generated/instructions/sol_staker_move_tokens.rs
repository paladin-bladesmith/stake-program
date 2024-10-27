//! This code was AUTOGENERATED using the kinobi library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun kinobi to update it.
//!
//! <https://github.com/kinobi-so/kinobi>
//!

use borsh::BorshDeserialize;
use borsh::BorshSerialize;

/// Accounts.
pub struct SolStakerMoveTokens {
    /// Staking config
    pub config: solana_program::pubkey::Pubkey,
    /// Vault holder rewards
    pub vault_holder_rewards: solana_program::pubkey::Pubkey,
    /// Sol staker authority
    pub sol_staker_authority: solana_program::pubkey::Pubkey,
    /// Source sol staker stake
    pub source_sol_staker_stake: solana_program::pubkey::Pubkey,
    /// Destination sol staker stake
    pub destination_sol_staker_stake: solana_program::pubkey::Pubkey,
}

impl SolStakerMoveTokens {
    pub fn instruction(
        &self,
        args: SolStakerMoveTokensInstructionArgs,
    ) -> solana_program::instruction::Instruction {
        self.instruction_with_remaining_accounts(args, &[])
    }
    #[allow(clippy::vec_init_then_push)]
    pub fn instruction_with_remaining_accounts(
        &self,
        args: SolStakerMoveTokensInstructionArgs,
        remaining_accounts: &[solana_program::instruction::AccountMeta],
    ) -> solana_program::instruction::Instruction {
        let mut accounts = Vec::with_capacity(5 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.config,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.vault_holder_rewards,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.sol_staker_authority,
            true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.source_sol_staker_stake,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.destination_sol_staker_stake,
            false,
        ));
        accounts.extend_from_slice(remaining_accounts);
        let mut data = SolStakerMoveTokensInstructionData::new()
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
pub struct SolStakerMoveTokensInstructionData {
    discriminator: u8,
}

impl SolStakerMoveTokensInstructionData {
    pub fn new() -> Self {
        Self { discriminator: 16 }
    }
}

impl Default for SolStakerMoveTokensInstructionData {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SolStakerMoveTokensInstructionArgs {
    pub amount: u64,
}

/// Instruction builder for `SolStakerMoveTokens`.
///
/// ### Accounts:
///
///   0. `[writable]` config
///   1. `[]` vault_holder_rewards
///   2. `[signer]` sol_staker_authority
///   3. `[writable]` source_sol_staker_stake
///   4. `[writable]` destination_sol_staker_stake
#[derive(Clone, Debug, Default)]
pub struct SolStakerMoveTokensBuilder {
    config: Option<solana_program::pubkey::Pubkey>,
    vault_holder_rewards: Option<solana_program::pubkey::Pubkey>,
    sol_staker_authority: Option<solana_program::pubkey::Pubkey>,
    source_sol_staker_stake: Option<solana_program::pubkey::Pubkey>,
    destination_sol_staker_stake: Option<solana_program::pubkey::Pubkey>,
    amount: Option<u64>,
    __remaining_accounts: Vec<solana_program::instruction::AccountMeta>,
}

impl SolStakerMoveTokensBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    /// Staking config
    #[inline(always)]
    pub fn config(&mut self, config: solana_program::pubkey::Pubkey) -> &mut Self {
        self.config = Some(config);
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
    /// Sol staker authority
    #[inline(always)]
    pub fn sol_staker_authority(
        &mut self,
        sol_staker_authority: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.sol_staker_authority = Some(sol_staker_authority);
        self
    }
    /// Source sol staker stake
    #[inline(always)]
    pub fn source_sol_staker_stake(
        &mut self,
        source_sol_staker_stake: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.source_sol_staker_stake = Some(source_sol_staker_stake);
        self
    }
    /// Destination sol staker stake
    #[inline(always)]
    pub fn destination_sol_staker_stake(
        &mut self,
        destination_sol_staker_stake: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.destination_sol_staker_stake = Some(destination_sol_staker_stake);
        self
    }
    #[inline(always)]
    pub fn amount(&mut self, amount: u64) -> &mut Self {
        self.amount = Some(amount);
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
        let accounts = SolStakerMoveTokens {
            config: self.config.expect("config is not set"),
            vault_holder_rewards: self
                .vault_holder_rewards
                .expect("vault_holder_rewards is not set"),
            sol_staker_authority: self
                .sol_staker_authority
                .expect("sol_staker_authority is not set"),
            source_sol_staker_stake: self
                .source_sol_staker_stake
                .expect("source_sol_staker_stake is not set"),
            destination_sol_staker_stake: self
                .destination_sol_staker_stake
                .expect("destination_sol_staker_stake is not set"),
        };
        let args = SolStakerMoveTokensInstructionArgs {
            amount: self.amount.clone().expect("amount is not set"),
        };

        accounts.instruction_with_remaining_accounts(args, &self.__remaining_accounts)
    }
}

/// `sol_staker_move_tokens` CPI accounts.
pub struct SolStakerMoveTokensCpiAccounts<'a, 'b> {
    /// Staking config
    pub config: &'b solana_program::account_info::AccountInfo<'a>,
    /// Vault holder rewards
    pub vault_holder_rewards: &'b solana_program::account_info::AccountInfo<'a>,
    /// Sol staker authority
    pub sol_staker_authority: &'b solana_program::account_info::AccountInfo<'a>,
    /// Source sol staker stake
    pub source_sol_staker_stake: &'b solana_program::account_info::AccountInfo<'a>,
    /// Destination sol staker stake
    pub destination_sol_staker_stake: &'b solana_program::account_info::AccountInfo<'a>,
}

/// `sol_staker_move_tokens` CPI instruction.
pub struct SolStakerMoveTokensCpi<'a, 'b> {
    /// The program to invoke.
    pub __program: &'b solana_program::account_info::AccountInfo<'a>,
    /// Staking config
    pub config: &'b solana_program::account_info::AccountInfo<'a>,
    /// Vault holder rewards
    pub vault_holder_rewards: &'b solana_program::account_info::AccountInfo<'a>,
    /// Sol staker authority
    pub sol_staker_authority: &'b solana_program::account_info::AccountInfo<'a>,
    /// Source sol staker stake
    pub source_sol_staker_stake: &'b solana_program::account_info::AccountInfo<'a>,
    /// Destination sol staker stake
    pub destination_sol_staker_stake: &'b solana_program::account_info::AccountInfo<'a>,
    /// The arguments for the instruction.
    pub __args: SolStakerMoveTokensInstructionArgs,
}

impl<'a, 'b> SolStakerMoveTokensCpi<'a, 'b> {
    pub fn new(
        program: &'b solana_program::account_info::AccountInfo<'a>,
        accounts: SolStakerMoveTokensCpiAccounts<'a, 'b>,
        args: SolStakerMoveTokensInstructionArgs,
    ) -> Self {
        Self {
            __program: program,
            config: accounts.config,
            vault_holder_rewards: accounts.vault_holder_rewards,
            sol_staker_authority: accounts.sol_staker_authority,
            source_sol_staker_stake: accounts.source_sol_staker_stake,
            destination_sol_staker_stake: accounts.destination_sol_staker_stake,
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
            *self.vault_holder_rewards.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.sol_staker_authority.key,
            true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.source_sol_staker_stake.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.destination_sol_staker_stake.key,
            false,
        ));
        remaining_accounts.iter().for_each(|remaining_account| {
            accounts.push(solana_program::instruction::AccountMeta {
                pubkey: *remaining_account.0.key,
                is_signer: remaining_account.1,
                is_writable: remaining_account.2,
            })
        });
        let mut data = SolStakerMoveTokensInstructionData::new()
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
        account_infos.push(self.vault_holder_rewards.clone());
        account_infos.push(self.sol_staker_authority.clone());
        account_infos.push(self.source_sol_staker_stake.clone());
        account_infos.push(self.destination_sol_staker_stake.clone());
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

/// Instruction builder for `SolStakerMoveTokens` via CPI.
///
/// ### Accounts:
///
///   0. `[writable]` config
///   1. `[]` vault_holder_rewards
///   2. `[signer]` sol_staker_authority
///   3. `[writable]` source_sol_staker_stake
///   4. `[writable]` destination_sol_staker_stake
#[derive(Clone, Debug)]
pub struct SolStakerMoveTokensCpiBuilder<'a, 'b> {
    instruction: Box<SolStakerMoveTokensCpiBuilderInstruction<'a, 'b>>,
}

impl<'a, 'b> SolStakerMoveTokensCpiBuilder<'a, 'b> {
    pub fn new(program: &'b solana_program::account_info::AccountInfo<'a>) -> Self {
        let instruction = Box::new(SolStakerMoveTokensCpiBuilderInstruction {
            __program: program,
            config: None,
            vault_holder_rewards: None,
            sol_staker_authority: None,
            source_sol_staker_stake: None,
            destination_sol_staker_stake: None,
            amount: None,
            __remaining_accounts: Vec::new(),
        });
        Self { instruction }
    }
    /// Staking config
    #[inline(always)]
    pub fn config(
        &mut self,
        config: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.config = Some(config);
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
    /// Sol staker authority
    #[inline(always)]
    pub fn sol_staker_authority(
        &mut self,
        sol_staker_authority: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.sol_staker_authority = Some(sol_staker_authority);
        self
    }
    /// Source sol staker stake
    #[inline(always)]
    pub fn source_sol_staker_stake(
        &mut self,
        source_sol_staker_stake: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.source_sol_staker_stake = Some(source_sol_staker_stake);
        self
    }
    /// Destination sol staker stake
    #[inline(always)]
    pub fn destination_sol_staker_stake(
        &mut self,
        destination_sol_staker_stake: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.destination_sol_staker_stake = Some(destination_sol_staker_stake);
        self
    }
    #[inline(always)]
    pub fn amount(&mut self, amount: u64) -> &mut Self {
        self.instruction.amount = Some(amount);
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
        let args = SolStakerMoveTokensInstructionArgs {
            amount: self.instruction.amount.clone().expect("amount is not set"),
        };
        let instruction = SolStakerMoveTokensCpi {
            __program: self.instruction.__program,

            config: self.instruction.config.expect("config is not set"),

            vault_holder_rewards: self
                .instruction
                .vault_holder_rewards
                .expect("vault_holder_rewards is not set"),

            sol_staker_authority: self
                .instruction
                .sol_staker_authority
                .expect("sol_staker_authority is not set"),

            source_sol_staker_stake: self
                .instruction
                .source_sol_staker_stake
                .expect("source_sol_staker_stake is not set"),

            destination_sol_staker_stake: self
                .instruction
                .destination_sol_staker_stake
                .expect("destination_sol_staker_stake is not set"),
            __args: args,
        };
        instruction.invoke_signed_with_remaining_accounts(
            signers_seeds,
            &self.instruction.__remaining_accounts,
        )
    }
}

#[derive(Clone, Debug)]
struct SolStakerMoveTokensCpiBuilderInstruction<'a, 'b> {
    __program: &'b solana_program::account_info::AccountInfo<'a>,
    config: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    vault_holder_rewards: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    sol_staker_authority: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    source_sol_staker_stake: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    destination_sol_staker_stake: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    amount: Option<u64>,
    /// Additional instruction accounts `(AccountInfo, is_writable, is_signer)`.
    __remaining_accounts: Vec<(
        &'b solana_program::account_info::AccountInfo<'a>,
        bool,
        bool,
    )>,
}
