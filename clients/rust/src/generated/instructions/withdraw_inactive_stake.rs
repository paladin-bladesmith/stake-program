//! This code was AUTOGENERATED using the kinobi library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun kinobi to update it.
//!
//! <https://github.com/kinobi-so/kinobi>
//!

use borsh::BorshDeserialize;
use borsh::BorshSerialize;

/// Accounts.
pub struct WithdrawInactiveStake {
    /// Stake config account
    pub config: solana_program::pubkey::Pubkey,
    /// Validator stake account (pda of `['stake::state::stake', validator, config]`)
    pub stake: solana_program::pubkey::Pubkey,
    /// Vault token account
    pub vault: solana_program::pubkey::Pubkey,
    /// Destination token account
    pub destination_token_account: solana_program::pubkey::Pubkey,
    /// Stake Token Mint
    pub mint: solana_program::pubkey::Pubkey,
    /// Stake authority
    pub stake_authority: solana_program::pubkey::Pubkey,
    /// Vault authority (pda of `['token-owner', config]`)
    pub vault_authority: solana_program::pubkey::Pubkey,
    /// Token program
    pub token_program: solana_program::pubkey::Pubkey,
}

impl WithdrawInactiveStake {
    pub fn instruction(
        &self,
        args: WithdrawInactiveStakeInstructionArgs,
    ) -> solana_program::instruction::Instruction {
        self.instruction_with_remaining_accounts(args, &[])
    }
    #[allow(clippy::vec_init_then_push)]
    pub fn instruction_with_remaining_accounts(
        &self,
        args: WithdrawInactiveStakeInstructionArgs,
        remaining_accounts: &[solana_program::instruction::AccountMeta],
    ) -> solana_program::instruction::Instruction {
        let mut accounts = Vec::with_capacity(8 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.config,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.stake, false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.vault, false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.destination_token_account,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.mint, false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.stake_authority,
            true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.vault_authority,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.token_program,
            false,
        ));
        accounts.extend_from_slice(remaining_accounts);
        let mut data = WithdrawInactiveStakeInstructionData::new()
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
pub struct WithdrawInactiveStakeInstructionData {
    discriminator: u8,
}

impl WithdrawInactiveStakeInstructionData {
    pub fn new() -> Self {
        Self { discriminator: 5 }
    }
}

impl Default for WithdrawInactiveStakeInstructionData {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WithdrawInactiveStakeInstructionArgs {
    pub amount: u64,
}

/// Instruction builder for `WithdrawInactiveStake`.
///
/// ### Accounts:
///
///   0. `[writable]` config
///   1. `[writable]` stake
///   2. `[writable]` vault
///   3. `[writable]` destination_token_account
///   4. `[]` mint
///   5. `[signer]` stake_authority
///   6. `[]` vault_authority
///   7. `[optional]` token_program (default to `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA`)
#[derive(Clone, Debug, Default)]
pub struct WithdrawInactiveStakeBuilder {
    config: Option<solana_program::pubkey::Pubkey>,
    stake: Option<solana_program::pubkey::Pubkey>,
    vault: Option<solana_program::pubkey::Pubkey>,
    destination_token_account: Option<solana_program::pubkey::Pubkey>,
    mint: Option<solana_program::pubkey::Pubkey>,
    stake_authority: Option<solana_program::pubkey::Pubkey>,
    vault_authority: Option<solana_program::pubkey::Pubkey>,
    token_program: Option<solana_program::pubkey::Pubkey>,
    amount: Option<u64>,
    __remaining_accounts: Vec<solana_program::instruction::AccountMeta>,
}

impl WithdrawInactiveStakeBuilder {
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
    /// Vault token account
    #[inline(always)]
    pub fn vault(&mut self, vault: solana_program::pubkey::Pubkey) -> &mut Self {
        self.vault = Some(vault);
        self
    }
    /// Destination token account
    #[inline(always)]
    pub fn destination_token_account(
        &mut self,
        destination_token_account: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.destination_token_account = Some(destination_token_account);
        self
    }
    /// Stake Token Mint
    #[inline(always)]
    pub fn mint(&mut self, mint: solana_program::pubkey::Pubkey) -> &mut Self {
        self.mint = Some(mint);
        self
    }
    /// Stake authority
    #[inline(always)]
    pub fn stake_authority(
        &mut self,
        stake_authority: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.stake_authority = Some(stake_authority);
        self
    }
    /// Vault authority (pda of `['token-owner', config]`)
    #[inline(always)]
    pub fn vault_authority(
        &mut self,
        vault_authority: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.vault_authority = Some(vault_authority);
        self
    }
    /// `[optional account, default to 'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA']`
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
        let accounts = WithdrawInactiveStake {
            config: self.config.expect("config is not set"),
            stake: self.stake.expect("stake is not set"),
            vault: self.vault.expect("vault is not set"),
            destination_token_account: self
                .destination_token_account
                .expect("destination_token_account is not set"),
            mint: self.mint.expect("mint is not set"),
            stake_authority: self.stake_authority.expect("stake_authority is not set"),
            vault_authority: self.vault_authority.expect("vault_authority is not set"),
            token_program: self.token_program.unwrap_or(solana_program::pubkey!(
                "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
            )),
        };
        let args = WithdrawInactiveStakeInstructionArgs {
            amount: self.amount.clone().expect("amount is not set"),
        };

        accounts.instruction_with_remaining_accounts(args, &self.__remaining_accounts)
    }
}

/// `withdraw_inactive_stake` CPI accounts.
pub struct WithdrawInactiveStakeCpiAccounts<'a, 'b> {
    /// Stake config account
    pub config: &'b solana_program::account_info::AccountInfo<'a>,
    /// Validator stake account (pda of `['stake::state::stake', validator, config]`)
    pub stake: &'b solana_program::account_info::AccountInfo<'a>,
    /// Vault token account
    pub vault: &'b solana_program::account_info::AccountInfo<'a>,
    /// Destination token account
    pub destination_token_account: &'b solana_program::account_info::AccountInfo<'a>,
    /// Stake Token Mint
    pub mint: &'b solana_program::account_info::AccountInfo<'a>,
    /// Stake authority
    pub stake_authority: &'b solana_program::account_info::AccountInfo<'a>,
    /// Vault authority (pda of `['token-owner', config]`)
    pub vault_authority: &'b solana_program::account_info::AccountInfo<'a>,
    /// Token program
    pub token_program: &'b solana_program::account_info::AccountInfo<'a>,
}

/// `withdraw_inactive_stake` CPI instruction.
pub struct WithdrawInactiveStakeCpi<'a, 'b> {
    /// The program to invoke.
    pub __program: &'b solana_program::account_info::AccountInfo<'a>,
    /// Stake config account
    pub config: &'b solana_program::account_info::AccountInfo<'a>,
    /// Validator stake account (pda of `['stake::state::stake', validator, config]`)
    pub stake: &'b solana_program::account_info::AccountInfo<'a>,
    /// Vault token account
    pub vault: &'b solana_program::account_info::AccountInfo<'a>,
    /// Destination token account
    pub destination_token_account: &'b solana_program::account_info::AccountInfo<'a>,
    /// Stake Token Mint
    pub mint: &'b solana_program::account_info::AccountInfo<'a>,
    /// Stake authority
    pub stake_authority: &'b solana_program::account_info::AccountInfo<'a>,
    /// Vault authority (pda of `['token-owner', config]`)
    pub vault_authority: &'b solana_program::account_info::AccountInfo<'a>,
    /// Token program
    pub token_program: &'b solana_program::account_info::AccountInfo<'a>,
    /// The arguments for the instruction.
    pub __args: WithdrawInactiveStakeInstructionArgs,
}

impl<'a, 'b> WithdrawInactiveStakeCpi<'a, 'b> {
    pub fn new(
        program: &'b solana_program::account_info::AccountInfo<'a>,
        accounts: WithdrawInactiveStakeCpiAccounts<'a, 'b>,
        args: WithdrawInactiveStakeInstructionArgs,
    ) -> Self {
        Self {
            __program: program,
            config: accounts.config,
            stake: accounts.stake,
            vault: accounts.vault,
            destination_token_account: accounts.destination_token_account,
            mint: accounts.mint,
            stake_authority: accounts.stake_authority,
            vault_authority: accounts.vault_authority,
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
        let mut accounts = Vec::with_capacity(8 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.config.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.stake.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.vault.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.destination_token_account.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.mint.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.stake_authority.key,
            true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.vault_authority.key,
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
        let mut data = WithdrawInactiveStakeInstructionData::new()
            .try_to_vec()
            .unwrap();
        let mut args = self.__args.try_to_vec().unwrap();
        data.append(&mut args);

        let instruction = solana_program::instruction::Instruction {
            program_id: crate::PALADIN_STAKE_PROGRAM_ID,
            accounts,
            data,
        };
        let mut account_infos = Vec::with_capacity(8 + 1 + remaining_accounts.len());
        account_infos.push(self.__program.clone());
        account_infos.push(self.config.clone());
        account_infos.push(self.stake.clone());
        account_infos.push(self.vault.clone());
        account_infos.push(self.destination_token_account.clone());
        account_infos.push(self.mint.clone());
        account_infos.push(self.stake_authority.clone());
        account_infos.push(self.vault_authority.clone());
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

/// Instruction builder for `WithdrawInactiveStake` via CPI.
///
/// ### Accounts:
///
///   0. `[writable]` config
///   1. `[writable]` stake
///   2. `[writable]` vault
///   3. `[writable]` destination_token_account
///   4. `[]` mint
///   5. `[signer]` stake_authority
///   6. `[]` vault_authority
///   7. `[]` token_program
#[derive(Clone, Debug)]
pub struct WithdrawInactiveStakeCpiBuilder<'a, 'b> {
    instruction: Box<WithdrawInactiveStakeCpiBuilderInstruction<'a, 'b>>,
}

impl<'a, 'b> WithdrawInactiveStakeCpiBuilder<'a, 'b> {
    pub fn new(program: &'b solana_program::account_info::AccountInfo<'a>) -> Self {
        let instruction = Box::new(WithdrawInactiveStakeCpiBuilderInstruction {
            __program: program,
            config: None,
            stake: None,
            vault: None,
            destination_token_account: None,
            mint: None,
            stake_authority: None,
            vault_authority: None,
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
    /// Validator stake account (pda of `['stake::state::stake', validator, config]`)
    #[inline(always)]
    pub fn stake(&mut self, stake: &'b solana_program::account_info::AccountInfo<'a>) -> &mut Self {
        self.instruction.stake = Some(stake);
        self
    }
    /// Vault token account
    #[inline(always)]
    pub fn vault(&mut self, vault: &'b solana_program::account_info::AccountInfo<'a>) -> &mut Self {
        self.instruction.vault = Some(vault);
        self
    }
    /// Destination token account
    #[inline(always)]
    pub fn destination_token_account(
        &mut self,
        destination_token_account: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.destination_token_account = Some(destination_token_account);
        self
    }
    /// Stake Token Mint
    #[inline(always)]
    pub fn mint(&mut self, mint: &'b solana_program::account_info::AccountInfo<'a>) -> &mut Self {
        self.instruction.mint = Some(mint);
        self
    }
    /// Stake authority
    #[inline(always)]
    pub fn stake_authority(
        &mut self,
        stake_authority: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.stake_authority = Some(stake_authority);
        self
    }
    /// Vault authority (pda of `['token-owner', config]`)
    #[inline(always)]
    pub fn vault_authority(
        &mut self,
        vault_authority: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.vault_authority = Some(vault_authority);
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
        let args = WithdrawInactiveStakeInstructionArgs {
            amount: self.instruction.amount.clone().expect("amount is not set"),
        };
        let instruction = WithdrawInactiveStakeCpi {
            __program: self.instruction.__program,

            config: self.instruction.config.expect("config is not set"),

            stake: self.instruction.stake.expect("stake is not set"),

            vault: self.instruction.vault.expect("vault is not set"),

            destination_token_account: self
                .instruction
                .destination_token_account
                .expect("destination_token_account is not set"),

            mint: self.instruction.mint.expect("mint is not set"),

            stake_authority: self
                .instruction
                .stake_authority
                .expect("stake_authority is not set"),

            vault_authority: self
                .instruction
                .vault_authority
                .expect("vault_authority is not set"),

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
struct WithdrawInactiveStakeCpiBuilderInstruction<'a, 'b> {
    __program: &'b solana_program::account_info::AccountInfo<'a>,
    config: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    stake: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    vault: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    destination_token_account: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    mint: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    stake_authority: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    vault_authority: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    token_program: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    amount: Option<u64>,
    /// Additional instruction accounts `(AccountInfo, is_writable, is_signer)`.
    __remaining_accounts: Vec<(
        &'b solana_program::account_info::AccountInfo<'a>,
        bool,
        bool,
    )>,
}
