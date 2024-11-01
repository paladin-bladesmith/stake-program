//! This code was AUTOGENERATED using the kinobi library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun kinobi to update it.
//!
//! <https://github.com/kinobi-so/kinobi>
//!

use borsh::BorshDeserialize;
use borsh::BorshSerialize;

/// Accounts.
pub struct ValidatorStakeTokens {
    /// Stake config account
    pub config: solana_program::pubkey::Pubkey,
    /// Validator stake account
    pub validator_stake: solana_program::pubkey::Pubkey,
    /// Validator stake account
    pub validator_stake_authority: solana_program::pubkey::Pubkey,
    /// Token account
    pub source_token_account: solana_program::pubkey::Pubkey,
    /// Owner or delegate of the token account
    pub source_token_account_authority: solana_program::pubkey::Pubkey,
    /// Stake Token Mint
    pub mint: solana_program::pubkey::Pubkey,
    /// Stake token Vault
    pub vault: solana_program::pubkey::Pubkey,
    /// Holder rewards for the vault account (to facilitate harvest)
    pub vault_holder_rewards: solana_program::pubkey::Pubkey,
    /// Token program
    pub token_program: solana_program::pubkey::Pubkey,
}

impl ValidatorStakeTokens {
    pub fn instruction(
        &self,
        args: ValidatorStakeTokensInstructionArgs,
    ) -> solana_program::instruction::Instruction {
        self.instruction_with_remaining_accounts(args, &[])
    }
    #[allow(clippy::vec_init_then_push)]
    pub fn instruction_with_remaining_accounts(
        &self,
        args: ValidatorStakeTokensInstructionArgs,
        remaining_accounts: &[solana_program::instruction::AccountMeta],
    ) -> solana_program::instruction::Instruction {
        let mut accounts = Vec::with_capacity(9 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.config,
            false,
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
            self.source_token_account,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.source_token_account_authority,
            true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.mint, false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.vault, false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.vault_holder_rewards,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.token_program,
            false,
        ));
        accounts.extend_from_slice(remaining_accounts);
        let mut data = ValidatorStakeTokensInstructionData::new()
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
pub struct ValidatorStakeTokensInstructionData {
    discriminator: u8,
}

impl ValidatorStakeTokensInstructionData {
    pub fn new() -> Self {
        Self { discriminator: 2 }
    }
}

impl Default for ValidatorStakeTokensInstructionData {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ValidatorStakeTokensInstructionArgs {
    pub amount: u64,
}

/// Instruction builder for `ValidatorStakeTokens`.
///
/// ### Accounts:
///
///   0. `[writable]` config
///   1. `[writable]` validator_stake
///   2. `[writable]` validator_stake_authority
///   3. `[writable]` source_token_account
///   4. `[signer]` source_token_account_authority
///   5. `[]` mint
///   6. `[writable]` vault
///   7. `[writable]` vault_holder_rewards
///   8. `[optional]` token_program (default to `TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb`)
#[derive(Clone, Debug, Default)]
pub struct ValidatorStakeTokensBuilder {
    config: Option<solana_program::pubkey::Pubkey>,
    validator_stake: Option<solana_program::pubkey::Pubkey>,
    validator_stake_authority: Option<solana_program::pubkey::Pubkey>,
    source_token_account: Option<solana_program::pubkey::Pubkey>,
    source_token_account_authority: Option<solana_program::pubkey::Pubkey>,
    mint: Option<solana_program::pubkey::Pubkey>,
    vault: Option<solana_program::pubkey::Pubkey>,
    vault_holder_rewards: Option<solana_program::pubkey::Pubkey>,
    token_program: Option<solana_program::pubkey::Pubkey>,
    amount: Option<u64>,
    __remaining_accounts: Vec<solana_program::instruction::AccountMeta>,
}

impl ValidatorStakeTokensBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    /// Stake config account
    #[inline(always)]
    pub fn config(&mut self, config: solana_program::pubkey::Pubkey) -> &mut Self {
        self.config = Some(config);
        self
    }
    /// Validator stake account
    #[inline(always)]
    pub fn validator_stake(
        &mut self,
        validator_stake: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.validator_stake = Some(validator_stake);
        self
    }
    /// Validator stake account
    #[inline(always)]
    pub fn validator_stake_authority(
        &mut self,
        validator_stake_authority: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.validator_stake_authority = Some(validator_stake_authority);
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
    pub fn source_token_account_authority(
        &mut self,
        source_token_account_authority: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.source_token_account_authority = Some(source_token_account_authority);
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
    /// Holder rewards for the vault account (to facilitate harvest)
    #[inline(always)]
    pub fn vault_holder_rewards(
        &mut self,
        vault_holder_rewards: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.vault_holder_rewards = Some(vault_holder_rewards);
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
        let accounts = ValidatorStakeTokens {
            config: self.config.expect("config is not set"),
            validator_stake: self.validator_stake.expect("validator_stake is not set"),
            validator_stake_authority: self
                .validator_stake_authority
                .expect("validator_stake_authority is not set"),
            source_token_account: self
                .source_token_account
                .expect("source_token_account is not set"),
            source_token_account_authority: self
                .source_token_account_authority
                .expect("source_token_account_authority is not set"),
            mint: self.mint.expect("mint is not set"),
            vault: self.vault.expect("vault is not set"),
            vault_holder_rewards: self
                .vault_holder_rewards
                .expect("vault_holder_rewards is not set"),
            token_program: self.token_program.unwrap_or(solana_program::pubkey!(
                "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"
            )),
        };
        let args = ValidatorStakeTokensInstructionArgs {
            amount: self.amount.clone().expect("amount is not set"),
        };

        accounts.instruction_with_remaining_accounts(args, &self.__remaining_accounts)
    }
}

/// `validator_stake_tokens` CPI accounts.
pub struct ValidatorStakeTokensCpiAccounts<'a, 'b> {
    /// Stake config account
    pub config: &'b solana_program::account_info::AccountInfo<'a>,
    /// Validator stake account
    pub validator_stake: &'b solana_program::account_info::AccountInfo<'a>,
    /// Validator stake account
    pub validator_stake_authority: &'b solana_program::account_info::AccountInfo<'a>,
    /// Token account
    pub source_token_account: &'b solana_program::account_info::AccountInfo<'a>,
    /// Owner or delegate of the token account
    pub source_token_account_authority: &'b solana_program::account_info::AccountInfo<'a>,
    /// Stake Token Mint
    pub mint: &'b solana_program::account_info::AccountInfo<'a>,
    /// Stake token Vault
    pub vault: &'b solana_program::account_info::AccountInfo<'a>,
    /// Holder rewards for the vault account (to facilitate harvest)
    pub vault_holder_rewards: &'b solana_program::account_info::AccountInfo<'a>,
    /// Token program
    pub token_program: &'b solana_program::account_info::AccountInfo<'a>,
}

/// `validator_stake_tokens` CPI instruction.
pub struct ValidatorStakeTokensCpi<'a, 'b> {
    /// The program to invoke.
    pub __program: &'b solana_program::account_info::AccountInfo<'a>,
    /// Stake config account
    pub config: &'b solana_program::account_info::AccountInfo<'a>,
    /// Validator stake account
    pub validator_stake: &'b solana_program::account_info::AccountInfo<'a>,
    /// Validator stake account
    pub validator_stake_authority: &'b solana_program::account_info::AccountInfo<'a>,
    /// Token account
    pub source_token_account: &'b solana_program::account_info::AccountInfo<'a>,
    /// Owner or delegate of the token account
    pub source_token_account_authority: &'b solana_program::account_info::AccountInfo<'a>,
    /// Stake Token Mint
    pub mint: &'b solana_program::account_info::AccountInfo<'a>,
    /// Stake token Vault
    pub vault: &'b solana_program::account_info::AccountInfo<'a>,
    /// Holder rewards for the vault account (to facilitate harvest)
    pub vault_holder_rewards: &'b solana_program::account_info::AccountInfo<'a>,
    /// Token program
    pub token_program: &'b solana_program::account_info::AccountInfo<'a>,
    /// The arguments for the instruction.
    pub __args: ValidatorStakeTokensInstructionArgs,
}

impl<'a, 'b> ValidatorStakeTokensCpi<'a, 'b> {
    pub fn new(
        program: &'b solana_program::account_info::AccountInfo<'a>,
        accounts: ValidatorStakeTokensCpiAccounts<'a, 'b>,
        args: ValidatorStakeTokensInstructionArgs,
    ) -> Self {
        Self {
            __program: program,
            config: accounts.config,
            validator_stake: accounts.validator_stake,
            validator_stake_authority: accounts.validator_stake_authority,
            source_token_account: accounts.source_token_account,
            source_token_account_authority: accounts.source_token_account_authority,
            mint: accounts.mint,
            vault: accounts.vault,
            vault_holder_rewards: accounts.vault_holder_rewards,
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
        let mut accounts = Vec::with_capacity(9 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.config.key,
            false,
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
            *self.source_token_account.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.source_token_account_authority.key,
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
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.vault_holder_rewards.key,
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
        let mut data = ValidatorStakeTokensInstructionData::new()
            .try_to_vec()
            .unwrap();
        let mut args = self.__args.try_to_vec().unwrap();
        data.append(&mut args);

        let instruction = solana_program::instruction::Instruction {
            program_id: crate::PALADIN_STAKE_PROGRAM_ID,
            accounts,
            data,
        };
        let mut account_infos = Vec::with_capacity(9 + 1 + remaining_accounts.len());
        account_infos.push(self.__program.clone());
        account_infos.push(self.config.clone());
        account_infos.push(self.validator_stake.clone());
        account_infos.push(self.validator_stake_authority.clone());
        account_infos.push(self.source_token_account.clone());
        account_infos.push(self.source_token_account_authority.clone());
        account_infos.push(self.mint.clone());
        account_infos.push(self.vault.clone());
        account_infos.push(self.vault_holder_rewards.clone());
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

/// Instruction builder for `ValidatorStakeTokens` via CPI.
///
/// ### Accounts:
///
///   0. `[writable]` config
///   1. `[writable]` validator_stake
///   2. `[writable]` validator_stake_authority
///   3. `[writable]` source_token_account
///   4. `[signer]` source_token_account_authority
///   5. `[]` mint
///   6. `[writable]` vault
///   7. `[writable]` vault_holder_rewards
///   8. `[]` token_program
#[derive(Clone, Debug)]
pub struct ValidatorStakeTokensCpiBuilder<'a, 'b> {
    instruction: Box<ValidatorStakeTokensCpiBuilderInstruction<'a, 'b>>,
}

impl<'a, 'b> ValidatorStakeTokensCpiBuilder<'a, 'b> {
    pub fn new(program: &'b solana_program::account_info::AccountInfo<'a>) -> Self {
        let instruction = Box::new(ValidatorStakeTokensCpiBuilderInstruction {
            __program: program,
            config: None,
            validator_stake: None,
            validator_stake_authority: None,
            source_token_account: None,
            source_token_account_authority: None,
            mint: None,
            vault: None,
            vault_holder_rewards: None,
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
    /// Validator stake account
    #[inline(always)]
    pub fn validator_stake(
        &mut self,
        validator_stake: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.validator_stake = Some(validator_stake);
        self
    }
    /// Validator stake account
    #[inline(always)]
    pub fn validator_stake_authority(
        &mut self,
        validator_stake_authority: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.validator_stake_authority = Some(validator_stake_authority);
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
    pub fn source_token_account_authority(
        &mut self,
        source_token_account_authority: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.source_token_account_authority = Some(source_token_account_authority);
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
    /// Holder rewards for the vault account (to facilitate harvest)
    #[inline(always)]
    pub fn vault_holder_rewards(
        &mut self,
        vault_holder_rewards: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.vault_holder_rewards = Some(vault_holder_rewards);
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
        let args = ValidatorStakeTokensInstructionArgs {
            amount: self.instruction.amount.clone().expect("amount is not set"),
        };
        let instruction = ValidatorStakeTokensCpi {
            __program: self.instruction.__program,

            config: self.instruction.config.expect("config is not set"),

            validator_stake: self
                .instruction
                .validator_stake
                .expect("validator_stake is not set"),

            validator_stake_authority: self
                .instruction
                .validator_stake_authority
                .expect("validator_stake_authority is not set"),

            source_token_account: self
                .instruction
                .source_token_account
                .expect("source_token_account is not set"),

            source_token_account_authority: self
                .instruction
                .source_token_account_authority
                .expect("source_token_account_authority is not set"),

            mint: self.instruction.mint.expect("mint is not set"),

            vault: self.instruction.vault.expect("vault is not set"),

            vault_holder_rewards: self
                .instruction
                .vault_holder_rewards
                .expect("vault_holder_rewards is not set"),

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
struct ValidatorStakeTokensCpiBuilderInstruction<'a, 'b> {
    __program: &'b solana_program::account_info::AccountInfo<'a>,
    config: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    validator_stake: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    validator_stake_authority: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    source_token_account: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    source_token_account_authority: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    mint: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    vault: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    vault_holder_rewards: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    token_program: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    amount: Option<u64>,
    /// Additional instruction accounts `(AccountInfo, is_writable, is_signer)`.
    __remaining_accounts: Vec<(
        &'b solana_program::account_info::AccountInfo<'a>,
        bool,
        bool,
    )>,
}
