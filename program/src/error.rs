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
