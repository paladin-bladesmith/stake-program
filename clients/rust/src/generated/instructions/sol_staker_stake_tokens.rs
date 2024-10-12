//! This code was AUTOGENERATED using the kinobi library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun kinobi to update it.
//!
//! <https://github.com/kinobi-so/kinobi>
//!

use borsh::BorshDeserialize;
use borsh::BorshSerialize;

/// Accounts.
pub struct SolStakerStakeTokens {
    /// Stake config account
    pub config: solana_program::pubkey::Pubkey,
    /// SOL staker stake account (pda of `['stake::state::sol_staker_stake', stake state, config]`)
    pub sol_staker_stake: solana_program::pubkey::Pubkey,
    /// Token account
    pub source_token_account: solana_program::pubkey::Pubkey,
    /// Owner or delegate of the token account
    pub token_account_authority: solana_program::pubkey::Pubkey,
    /// Stake Token Mint
    pub mint: solana_program::pubkey::Pubkey,
    /// Stake token Vault
    pub vault: solana_program::pubkey::Pubkey,
    /// Token program
    pub token_program: solana_program::pubkey::Pubkey,
}

impl SolStakerStakeTokens {
    pub fn instruction(
        &self,
        args: SolStakerStakeTokensInstructionArgs,
    ) -> solana_program::instruction::Instruction {
        self.instruction_with_remaining_accounts(args, &[])
    }
    #[allow(clippy::vec_init_then_push)]
    pub fn instruction_with_remaining_accounts(
        &self,
        args: SolStakerStakeTokensInstructionArgs,
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
            self.source_token_account,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.token_account_authority,
            true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.mint, false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.vault, false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.token_program,
            false,
        ));
        accounts.extend_from_slice(remaining_accounts);
        let mut data = SolStakerStakeTokensInstructionData::new()
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
pub struct SolStakerStakeTokensInstructionData {
    discriminator: u8,
}

impl SolStakerStakeTokensInstructionData {
    pub fn new() -> Self {
        Self { discriminator: 13 }
    }
}

impl Default for SolStakerStakeTokensInstructionData {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SolStakerStakeTokensInstructionArgs {
    pub amount: u64,
}

/// Instruction builder for `SolStakerStakeTokens`.
///
/// ### Accounts:
///
///   0. `[writable]` config
///   1. `[writable]` sol_staker_stake
///   2. `[writable]` source_token_account
///   3. `[signer]` token_account_authority
///   4. `[]` mint
///   5. `[writable]` vault
///   6. `[optional]` token_program (default to `TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb`)
#[derive(Clone, Debug, Default)]
pub struct SolStakerStakeTokensBuilder {
    config: Option<solana_program::pubkey::Pubkey>,
    sol_staker_stake: Option<solana_program::pubkey::Pubkey>,
    source_token_account: Option<solana_program::pubkey::Pubkey>,
    token_account_authority: Option<solana_program::pubkey::Pubkey>,
    mint: Option<solana_program::pubkey::Pubkey>,
    vault: Option<solana_program::pubkey::Pubkey>,
    token_program: Option<solana_program::pubkey::Pubkey>,
    amount: Option<u64>,
    __remaining_accounts: Vec<solana_program::instruction::AccountMeta>,
}

impl SolStakerStakeTokensBuilder {
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
    /// Token account
    #[inline(always)]
    pub fn source_token_account(
        &mut self,
        source_token_account: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.source_token_account = Some(source_token_account);
        self
    }
    /// Owner or delegate of the token account
    #[inline(always)]
    pub fn token_account_authority(
        &mut self,
        token_account_authority: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.token_account_authority = Some(token_account_authority);
        self
    }
    /// Stake Token Mint
    #[inline(always)]
    pub fn mint(&mut self, mint: solana_program::pubkey::Pubkey) -> &mut Self {
        self.mint = Some(mint);
        self
    }
    /// Stake token Vault
    #[inline(always)]
    pub fn vault(&mut self, vault: solana_program::pubkey::Pubkey) -> &mut Self {
        self.vault = Some(vault);
        self
    }
    /// `[optional account, default to 'TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb']`
    /// Token program
    #[inline(always)]
    pub fn token_program(&mut self, token_program: solana_program::pubkey::Pubkey) -> &mut Self {
        self.token_program = Some(token_program);
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
        let accounts = SolStakerStakeTokens {
            config: self.config.expect("config is not set"),
            sol_staker_stake: self.sol_staker_stake.expect("sol_staker_stake is not set"),
            source_token_account: self
                .source_token_account
                .expect("source_token_account is not set"),
            token_account_authority: self
                .token_account_authority
                .expect("token_account_authority is not set"),
            mint: self.mint.expect("mint is not set"),
            vault: self.vault.expect("vault is not set"),
            token_program: self.token_program.unwrap_or(solana_program::pubkey!(
                "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"
            )),
        };
        let args = SolStakerStakeTokensInstructionArgs {
            amount: self.amount.clone().expect("amount is not set"),
        };

        accounts.instruction_with_remaining_accounts(args, &self.__remaining_accounts)
    }
}

/// `sol_staker_stake_tokens` CPI accounts.
pub struct SolStakerStakeTokensCpiAccounts<'a, 'b> {
    /// Stake config account
    pub config: &'b solana_program::account_info::AccountInfo<'a>,
    /// SOL staker stake account (pda of `['stake::state::sol_staker_stake', stake state, config]`)
    pub sol_staker_stake: &'b solana_program::account_info::AccountInfo<'a>,
    /// Token account
    pub source_token_account: &'b solana_program::account_info::AccountInfo<'a>,
    /// Owner or delegate of the token account
    pub token_account_authority: &'b solana_program::account_info::AccountInfo<'a>,
    /// Stake Token Mint
    pub mint: &'b solana_program::account_info::AccountInfo<'a>,
    /// Stake token Vault
    pub vault: &'b solana_program::account_info::AccountInfo<'a>,
    /// Token program
    pub token_program: &'b solana_program::account_info::AccountInfo<'a>,
}

/// `sol_staker_stake_tokens` CPI instruction.
pub struct SolStakerStakeTokensCpi<'a, 'b> {
    /// The program to invoke.
    pub __program: &'b solana_program::account_info::AccountInfo<'a>,
    /// Stake config account
    pub config: &'b solana_program::account_info::AccountInfo<'a>,
    /// SOL staker stake account (pda of `['stake::state::sol_staker_stake', stake state, config]`)
    pub sol_staker_stake: &'b solana_program::account_info::AccountInfo<'a>,
    /// Token account
    pub source_token_account: &'b solana_program::account_info::AccountInfo<'a>,
    /// Owner or delegate of the token account
    pub token_account_authority: &'b solana_program::account_info::AccountInfo<'a>,
    /// Stake Token Mint
    pub mint: &'b solana_program::account_info::AccountInfo<'a>,
    /// Stake token Vault
    pub vault: &'b solana_program::account_info::AccountInfo<'a>,
    /// Token program
    pub token_program: &'b solana_program::account_info::AccountInfo<'a>,
    /// The arguments for the instruction.
    pub __args: SolStakerStakeTokensInstructionArgs,
}

impl<'a, 'b> SolStakerStakeTokensCpi<'a, 'b> {
    pub fn new(
        program: &'b solana_program::account_info::AccountInfo<'a>,
        accounts: SolStakerStakeTokensCpiAccounts<'a, 'b>,
        args: SolStakerStakeTokensInstructionArgs,
    ) -> Self {
        Self {
            __program: program,
            config: accounts.config,
            sol_staker_stake: accounts.sol_staker_stake,
            source_token_account: accounts.source_token_account,
            token_account_authority: accounts.token_account_authority,
            mint: accounts.mint,
            vault: accounts.vault,
            token_program: accounts.token_program,
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
            *self.source_token_account.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.token_account_authority.key,
            true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.mint.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.vault.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.token_program.key,
            false,
        ));
        remaining_accounts.iter().for_each(|remaining_account| {
            accounts.push(solana_program::instruction::AccountMeta {
                pubkey: *remaining_account.0.key,
                is_signer: remaining_account.1,
                is_writable: remaining_account.2,
            })
        });
        let mut data = SolStakerStakeTokensInstructionData::new()
            .try_to_vec()
            .unwrap();
        let mut args = self.__args.try_to_vec().unwrap();
        data.append(&mut args);

        let instruction = solana_program::instruction::Instruction {
            program_id: crate::PALADIN_STAKE_PROGRAM_ID,
            accounts,
            data,
        };
        let mut account_infos = Vec::with_capacity(7 + 1 + remaining_accounts.len());
        account_infos.push(self.__program.clone());
        account_infos.push(self.config.clone());
        account_infos.push(self.sol_staker_stake.clone());
        account_infos.push(self.source_token_account.clone());
        account_infos.push(self.token_account_authority.clone());
        account_infos.push(self.mint.clone());
        account_infos.push(self.vault.clone());
        account_infos.push(self.token_program.clone());
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

/// Instruction builder for `SolStakerStakeTokens` via CPI.
///
/// ### Accounts:
///
///   0. `[writable]` config
///   1. `[writable]` sol_staker_stake
///   2. `[writable]` source_token_account
///   3. `[signer]` token_account_authority
///   4. `[]` mint
///   5. `[writable]` vault
///   6. `[]` token_program
#[derive(Clone, Debug)]
pub struct SolStakerStakeTokensCpiBuilder<'a, 'b> {
    instruction: Box<SolStakerStakeTokensCpiBuilderInstruction<'a, 'b>>,
}

impl<'a, 'b> SolStakerStakeTokensCpiBuilder<'a, 'b> {
    pub fn new(program: &'b solana_program::account_info::AccountInfo<'a>) -> Self {
        let instruction = Box::new(SolStakerStakeTokensCpiBuilderInstruction {
            __program: program,
            config: None,
            sol_staker_stake: None,
            source_token_account: None,
            token_account_authority: None,
            mint: None,
            vault: None,
            token_program: None,
            amount: None,
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
    /// Token account
    #[inline(always)]
    pub fn source_token_account(
        &mut self,
        source_token_account: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.source_token_account = Some(source_token_account);
        self
    }
    /// Owner or delegate of the token account
    #[inline(always)]
    pub fn token_account_authority(
        &mut self,
        token_account_authority: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.token_account_authority = Some(token_account_authority);
        self
    }
    /// Stake Token Mint
    #[inline(always)]
    pub fn mint(&mut self, mint: &'b solana_program::account_info::AccountInfo<'a>) -> &mut Self {
        self.instruction.mint = Some(mint);
        self
    }
    /// Stake token Vault
    #[inline(always)]
    pub fn vault(&mut self, vault: &'b solana_program::account_info::AccountInfo<'a>) -> &mut Self {
        self.instruction.vault = Some(vault);
        self
    }
    /// Token program
    #[inline(always)]
    pub fn token_program(
        &mut self,
        token_program: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.token_program = Some(token_program);
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
        let args = SolStakerStakeTokensInstructionArgs {
            amount: self.instruction.amount.clone().expect("amount is not set"),
        };
        let instruction = SolStakerStakeTokensCpi {
            __program: self.instruction.__program,

            config: self.instruction.config.expect("config is not set"),

            sol_staker_stake: self
                .instruction
                .sol_staker_stake
                .expect("sol_staker_stake is not set"),

            source_token_account: self
                .instruction
                .source_token_account
                .expect("source_token_account is not set"),

            token_account_authority: self
                .instruction
                .token_account_authority
                .expect("token_account_authority is not set"),

            mint: self.instruction.mint.expect("mint is not set"),

            vault: self.instruction.vault.expect("vault is not set"),

            token_program: self
                .instruction
                .token_program
                .expect("token_program is not set"),
            __args: args,
        };
        instruction.invoke_signed_with_remaining_accounts(
            signers_seeds,
            &self.instruction.__remaining_accounts,
        )
    }
}

#[derive(Clone, Debug)]
struct SolStakerStakeTokensCpiBuilderInstruction<'a, 'b> {
    __program: &'b solana_program::account_info::AccountInfo<'a>,
    config: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    sol_staker_stake: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    source_token_account: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    token_account_authority: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    mint: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    vault: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    token_program: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    amount: Option<u64>,
    /// Additional instruction accounts `(AccountInfo, is_writable, is_signer)`.
    __remaining_accounts: Vec<(
        &'b solana_program::account_info::AccountInfo<'a>,
        bool,
        bool,
    )>,
}
