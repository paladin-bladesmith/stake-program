#![allow(non_local_definitions)]

use num_derive::FromPrimitive;
use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

// Note: Shank does not export the type when we use `spl_program_error`.
#[derive(Error, Clone, Debug, Eq, PartialEq, FromPrimitive)]
pub enum StakeError {
    /// 0 - Amount cannot be greater than zero
    #[error("Amount cannot be greater than zero")]
    AmountGreaterThanZero,

    /// 1 - Invalid token owner
    #[error("Invalid token owner")]
    InvalidTokenOwner,

    /// 2 - Invalid transfer hook program id
    #[error("Invalid transfer hook program id")]
    InvalidTransferHookProgramId,

    /// 3 - Invalid account data length
    #[error("Invalid account data length")]
    InvalidAccountDataLength,

    /// 4 - Invalid mint
    #[error("Invalid mint")]
    InvalidMint,

    /// 5 - Missing transfer hook
    #[error("Missing transfer hook")]
    MissingTransferHook,

    /// 6 - Close authority must be none
    #[error("Close authority must be none")]
    CloseAuthorityNotNone,

    /// 7 - Delegate must be none
    #[error("Delegate must be none")]
    DelegateNotNone,

    /// 8 - Invalid token account extension
    #[error("Invalid token account extension")]
    InvalidTokenAccountExtension,

    /// 9 - Invalid authority
    #[error("Invalid authority")]
    InvalidAuthority,

    /// 10 - Authority is not set
    #[error("Authority is not set")]
    AuthorityNotSet,

    /// 11 - Amount greater than stake amount
    #[error("Amount greater than stake amount")]
    InsufficientStakeAmount,

    /// 12 - Amount should be greater than 0
    #[error("Amount should be greater than 0")]
    InvalidAmount,

    /// 13 - Amount exeeds maximum deactivation amount
    #[error("Amount exeeds maximum deactivation amount")]
    MaximumDeactivationAmountExceeded,

    /// 14 - Active unstake cooldown
    #[error("Active unstake cooldown")]
    ActiveUnstakeCooldown,

    /// 15 - Incorrect vault account
    #[error("Incorrect vault account")]
    IncorrectVaultAccount,

    /// 16 - Invalid destination account
    #[error("Invalid destination account")]
    InvalidDestinationAccount,

    /// 17 - Invalid slash amount
    #[error("Invalid slash amount")]
    InvalidSlashAmount,

    /// 18 - Undelegated SOL stake account
    #[error("Undelegated SOL stake account")]
    UndelegatedSolStakeAccount,

    /// 19 - Total stake amount exceeds SOL limit
    #[error("Total stake amount exceeds SOL limit")]
    TotalStakeAmountExceedsSolLimit,

    /// 20 - Incorrect SOL stake account
    #[error("Incorrect SOL stake account")]
    IncorrectSolStakeAccount,

    /// 21 - Invalid holder rewards.
    #[error("Invalid holder rewards")]
    InvalidHolderRewards,

    /// 22 - DUNA document is not initialized
    #[error("DUNA document is not initialized")]
    DunaDocumentNotInitialized,

    /// 23 - Incorrect vault PDA account
    #[error("Incorrect vault PDA account")]
    IncorrectVaultPdaAccount,

    /// 24 - Invalid vault pda owner
    #[error("Invalid vault pda  owner")]
    InvalidVaultPdaOwner,
}

impl PrintProgramError for StakeError {
    fn print<E>(&self) {
        msg!(&self.to_string());
    }
}

impl From<StakeError> for ProgramError {
    fn from(e: StakeError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for StakeError {
    fn type_of() -> &'static str {
        "StakeError"
    }
}

#[macro_export]
macro_rules! err {
    ( $error:expr ) => {{
        Err($error.into())
    }};
    ( $error:expr, $msg:expr ) => {{
        solana_program::msg!("Log: {}", $msg);
        Err($error.into())
    }};
    ( $error:expr, $msg:literal, $($args:tt)+ ) => {{
        err!($error, &format!($msg, $($args)+))
    }};
}
