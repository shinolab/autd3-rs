use derive_more::Display;
use thiserror::Error;

#[derive(Error, Debug, Display, PartialEq, Clone)]
#[display("{}", msg)]
/// An error occurred during gain calculation.
pub struct GainError {
    msg: String,
}

impl GainError {
    /// Creates a new [`GainError`].
    pub fn new(msg: impl ToString) -> Self {
        Self {
            msg: msg.to_string(),
        }
    }
}
