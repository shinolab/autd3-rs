use autd3_core::link::LinkError;
use autd3_driver::error::AUTDDriverError;
use thiserror::Error;

/// A interface for error handling in autd3.
#[derive(Error, Debug, PartialEq)]
#[non_exhaustive]
pub enum AUTDError {
    /// Driver error.
    #[error("{0}")]
    Driver(#[from] AUTDDriverError),

    /// Unknown group key.
    #[error("Unknown group key({0})")]
    UnknownKey(String),
    /// Unused group key.
    #[error("Unused group key({0})")]
    UnusedKey(String),
}

impl From<LinkError> for AUTDError {
    fn from(e: LinkError) -> Self {
        AUTDError::Driver(AUTDDriverError::Link(e))
    }
}
