//! This code was AUTOGENERATED using the kinobi library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun kinobi to update it.
//!
//! <https://github.com/kinobi-so/kinobi>
//!

use num_derive::FromPrimitive;
use thiserror::Error;

#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum PaladinStakeProgramError {
    /// 0 - Amount cannot be greater than zero
    #[error("Amount cannot be greater than zero")]
    AmountGreaterThanZero = 0x0,
    /// 1 - Invalid token owner
    #[error("Invalid token owner")]
    InvalidTokenOwner = 0x1,
    /// 2 - Invalid transfer hook program id
    #[error("Invalid transfer hook program id")]
    InvalidTransferHookProgramId = 0x2,
    /// 3 - Invalid account data length
    #[error("Invalid account data length")]
    InvalidAccountDataLength = 0x3,
    /// 4 - Invalid mint
    #[error("Invalid mint")]
    InvalidMint = 0x4,
    /// 5 - Missing transfer hook
    #[error("Missing transfer hook")]
    MissingTransferHook = 0x5,
    /// 6 - Close authority must be none
    #[error("Close authority must be none")]
    CloseAuthorityNotNone = 0x6,
    /// 7 - Delegate must be none
    #[error("Delegate must be none")]
    DelegateNotNone = 0x7,
    /// 8 - Invalid token account extension
    #[error("Invalid token account extension")]
    InvalidTokenAccountExtension = 0x8,
    /// 9 - Invalid authority
    #[error("Invalid authority")]
    InvalidAuthority = 0x9,
    /// 10 - Authority is not set
    #[error("Authority is not set")]
    AuthorityNotSet = 0xA,
    /// 11 - Amount greater than stake amount
    #[error("Amount greater than stake amount")]
    InsufficientStakeAmount = 0xB,
    /// 12 - Amount should be greater than 0
    #[error("Amount should be greater than 0")]
    InvalidAmount = 0xC,
    /// 13 - Amount exeeds maximum deactivation amount
    #[error("Amount exeeds maximum deactivation amount")]
    MaximumDeactivationAmountExceeded = 0xD,
    /// 14 - Active deactivation cooldown
    #[error("Active deactivation cooldown")]
    ActiveDeactivationCooldown = 0xE,
    /// 15 - No deactivated tokens
    #[error("No deactivated tokens")]
    NoDeactivatedTokens = 0xF,
    /// 16 - Incorrect vault account
    #[error("Incorrect vault account")]
    IncorrectVaultAccount = 0x10,
    /// 17 - Not enough inactivated tokens
    #[error("Not enough inactivated tokens")]
    NotEnoughInactivatedTokens = 0x11,
    /// 18 - Invalid destination account
    #[error("Invalid destination account")]
    InvalidDestinationAccount = 0x12,
    /// 19 - Invalid slash amount
    #[error("Invalid slash amount")]
    InvalidSlashAmount = 0x13,
    /// 20 - Undelegated stake state account
    #[error("Undelegated stake state account")]
    UndelegatedStakeStateAccount = 0x14,
}

impl solana_program::program_error::PrintProgramError for PaladinStakeProgramError {
    fn print<E>(&self) {
        solana_program::msg!(&self.to_string());
    }
}
