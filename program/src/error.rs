use spl_program_error::spl_program_error;

#[spl_program_error]
pub enum StakeError {
    /// Placeholder error.
    #[error("Placeholder error")]
    PlaceholderError,
}

#[macro_export]
macro_rules! err {
    ( $error:expr ) => {{
        Err($error.into())
    }};
    ( $error:expr, $msg:expr ) => {{
        solana_program::msg!("[ERROR] {}", $msg);
        Err($error.into())
    }};
    ( $error:expr, $msg:literal, $($args:tt)+ ) => {{
        err!($error, &format!($msg, $($args)+))
    }};
}
