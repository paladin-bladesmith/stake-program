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

    /// 14 - Active deactivation cooldown
    #[error("Active deactivation cooldown")]
    ActiveDeactivationCooldown,

    /// 16 - No deactivated tokens
    #[error("No deactivated tokens")]
    NoDeactivatedTokens,
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
