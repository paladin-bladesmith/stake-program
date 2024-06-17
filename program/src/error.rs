use spl_program_error::spl_program_error;

#[spl_program_error]
pub enum StakeError {
    /// Placeholder error.
    #[error("Placeholder error")]
    PlaceholderError,
}
