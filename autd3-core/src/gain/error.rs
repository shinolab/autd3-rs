use alloc::string::{String, ToString};
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Clone)]
#[error("{msg}")]
/// An error occurred during gain calculation.
pub struct GainError {
    msg: String,
}

impl GainError {
    /// Creates a new [`GainError`].
    #[must_use]
    pub fn new(msg: impl ToString) -> Self {
        Self {
            msg: msg.to_string(),
        }
    }
}
