//! This code was AUTOGENERATED using the kinobi library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun kinobi to update it.
//!
//! <https://github.com/kinobi-so/kinobi>

use borsh::{BorshDeserialize, BorshSerialize};

/// Accounts.
pub struct HarvestStakeRewards {
    /// Stake config account
    pub config: solana_program::pubkey::Pubkey,
    /// Validator stake account (pda of `['stake::state::stake', validator,
    /// config]`)
    pub stake: solana_program::pubkey::Pubkey,
    /// Destination account for withdrawn lamports
    pub destination: solana_program::pubkey::Pubkey,
    /// Stake authority
    pub stake_authority: solana_program::pubkey::Pubkey,
}

impl HarvestStakeRewards {
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        self.instruction_with_remaining_accounts(&[])
    }
    #[allow(clippy::vec_init_then_push)]
    pub fn instruction_with_remaining_accounts(
        &self,
        remaining_accounts: &[solana_program::instruction::AccountMeta],
    ) -> solana_program::instruction::Instruction {
        let mut accounts = Vec::with_capacity(4 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.config,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.stake, false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.destination,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.stake_authority,
            true,
        ));
        accounts.extend_from_slice(remaining_accounts);
        let data = HarvestStakeRewardsInstructionData::new()
            .try_to_vec()
            .unwrap();

        solana_program::instruction::Instruction {
            program_id: crate::STAKE_ID,
            accounts,
            data,
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct HarvestStakeRewardsInstructionData {
    discriminator: u8,
}

impl HarvestStakeRewardsInstructionData {
    pub fn new() -> Self {
        Self { discriminator: 7 }
    }
}

impl Default for HarvestStakeRewardsInstructionData {
    fn default() -> Self {
        Self::new()
    }
}

/// Instruction builder for `HarvestStakeRewards`.
///
/// ### Accounts:
///
///   0. `[writable]` config
///   1. `[writable]` stake
///   2. `[writable]` destination
///   3. `[signer]` stake_authority
#[derive(Clone, Debug, Default)]
pub struct HarvestStakeRewardsBuilder {
    config: Option<solana_program::pubkey::Pubkey>,
    stake: Option<solana_program::pubkey::Pubkey>,
    destination: Option<solana_program::pubkey::Pubkey>,
    stake_authority: Option<solana_program::pubkey::Pubkey>,
    __remaining_accounts: Vec<solana_program::instruction::AccountMeta>,
}

impl HarvestStakeRewardsBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    /// Stake config account
    #[inline(always)]
    pub fn config(&mut self, config: solana_program::pubkey::Pubkey) -> &mut Self {
        self.config = Some(config);
        self
    }
    /// Validator stake account (pda of `['stake::state::stake', validator,
    /// config]`)
    #[inline(always)]
    pub fn stake(&mut self, stake: solana_program::pubkey::Pubkey) -> &mut Self {
        self.stake = Some(stake);
        self
    }
    /// Destination account for withdrawn lamports
    #[inline(always)]
    pub fn destination(&mut self, destination: solana_program::pubkey::Pubkey) -> &mut Self {
        self.destination = Some(destination);
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
        let accounts = HarvestStakeRewards {
            config: self.config.expect("config is not set"),
            stake: self.stake.expect("stake is not set"),
            destination: self.destination.expect("destination is not set"),
            stake_authority: self.stake_authority.expect("stake_authority is not set"),
        };

        accounts.instruction_with_remaining_accounts(&self.__remaining_accounts)
    }
}

/// `harvest_stake_rewards` CPI accounts.
pub struct HarvestStakeRewardsCpiAccounts<'a, 'b> {
    /// Stake config account
    pub config: &'b solana_program::account_info::AccountInfo<'a>,
    /// Validator stake account (pda of `['stake::state::stake', validator,
    /// config]`)
    pub stake: &'b solana_program::account_info::AccountInfo<'a>,
    /// Destination account for withdrawn lamports
    pub destination: &'b solana_program::account_info::AccountInfo<'a>,
    /// Stake authority
    pub stake_authority: &'b solana_program::account_info::AccountInfo<'a>,
}

/// `harvest_stake_rewards` CPI instruction.
pub struct HarvestStakeRewardsCpi<'a, 'b> {
    /// The program to invoke.
    pub __program: &'b solana_program::account_info::AccountInfo<'a>,
    /// Stake config account
    pub config: &'b solana_program::account_info::AccountInfo<'a>,
    /// Validator stake account (pda of `['stake::state::stake', validator,
    /// config]`)
    pub stake: &'b solana_program::account_info::AccountInfo<'a>,
    /// Destination account for withdrawn lamports
    pub destination: &'b solana_program::account_info::AccountInfo<'a>,
    /// Stake authority
    pub stake_authority: &'b solana_program::account_info::AccountInfo<'a>,
}

impl<'a, 'b> HarvestStakeRewardsCpi<'a, 'b> {
    pub fn new(
        program: &'b solana_program::account_info::AccountInfo<'a>,
        accounts: HarvestStakeRewardsCpiAccounts<'a, 'b>,
    ) -> Self {
        Self {
            __program: program,
            config: accounts.config,
            stake: accounts.stake,
            destination: accounts.destination,
            stake_authority: accounts.stake_authority,
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
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.config.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.stake.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.destination.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.stake_authority.key,
            true,
        ));
        remaining_accounts.iter().for_each(|remaining_account| {
            accounts.push(solana_program::instruction::AccountMeta {
                pubkey: *remaining_account.0.key,
                is_signer: remaining_account.1,
                is_writable: remaining_account.2,
            })
        });
        let data = HarvestStakeRewardsInstructionData::new()
            .try_to_vec()
            .unwrap();

        let instruction = solana_program::instruction::Instruction {
            program_id: crate::STAKE_ID,
            accounts,
            data,
        };
        let mut account_infos = Vec::with_capacity(4 + 1 + remaining_accounts.len());
        account_infos.push(self.__program.clone());
        account_infos.push(self.config.clone());
        account_infos.push(self.stake.clone());
        account_infos.push(self.destination.clone());
        account_infos.push(self.stake_authority.clone());
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

/// Instruction builder for `HarvestStakeRewards` via CPI.
///
/// ### Accounts:
///
///   0. `[writable]` config
///   1. `[writable]` stake
///   2. `[writable]` destination
///   3. `[signer]` stake_authority
#[derive(Clone, Debug)]
pub struct HarvestStakeRewardsCpiBuilder<'a, 'b> {
    instruction: Box<HarvestStakeRewardsCpiBuilderInstruction<'a, 'b>>,
}

impl<'a, 'b> HarvestStakeRewardsCpiBuilder<'a, 'b> {
    pub fn new(program: &'b solana_program::account_info::AccountInfo<'a>) -> Self {
        let instruction = Box::new(HarvestStakeRewardsCpiBuilderInstruction {
            __program: program,
            config: None,
            stake: None,
            destination: None,
            stake_authority: None,
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
    /// Validator stake account (pda of `['stake::state::stake', validator,
    /// config]`)
    #[inline(always)]
    pub fn stake(&mut self, stake: &'b solana_program::account_info::AccountInfo<'a>) -> &mut Self {
        self.instruction.stake = Some(stake);
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
    /// Stake authority
    #[inline(always)]
    pub fn stake_authority(
        &mut self,
        stake_authority: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.stake_authority = Some(stake_authority);
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
    /// Each account is represented by a tuple of the `AccountInfo`, a `bool`
    /// indicating whether the account is writable or not, and a `bool`
    /// indicating whether the account is a signer or not.
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
        let instruction = HarvestStakeRewardsCpi {
            __program: self.instruction.__program,

            config: self.instruction.config.expect("config is not set"),

            stake: self.instruction.stake.expect("stake is not set"),

            destination: self
                .instruction
                .destination
                .expect("destination is not set"),

            stake_authority: self
                .instruction
                .stake_authority
                .expect("stake_authority is not set"),
        };
        instruction.invoke_signed_with_remaining_accounts(
            signers_seeds,
            &self.instruction.__remaining_accounts,
        )
    }
}

#[derive(Clone, Debug)]
struct HarvestStakeRewardsCpiBuilderInstruction<'a, 'b> {
    __program: &'b solana_program::account_info::AccountInfo<'a>,
    config: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    stake: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    destination: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    stake_authority: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    /// Additional instruction accounts `(AccountInfo, is_writable, is_signer)`.
    __remaining_accounts: Vec<(
        &'b solana_program::account_info::AccountInfo<'a>,
        bool,
        bool,
    )>,
}
