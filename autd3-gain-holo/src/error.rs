use autd3_core::gain::GainError;
use thiserror::Error;

/// A interface for error handling in autd3-gain-holo.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum HoloError {
    #[error("Failed to solve linear system")]
    /// Failed to solve linear system.
    SolveFailed,
    #[error("{0}")]
    /// Backend error.
    BackendError(String),
    #[error("Invalid operation")]
    /// Invalid operation.
    InvalidOperation,
}

impl From<HoloError> for GainError {
    fn from(value: HoloError) -> Self {
        GainError::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn solve_failed() {
        let err = HoloError::SolveFailed;
        assert!(err.source().is_none());
        assert_eq!(format!("{}", err), "Failed to solve linear system");
        assert_eq!(format!("{:?}", err), "SolveFailed");
    }

    #[test]
    fn backend_error() {
        let err = HoloError::BackendError("test".to_string());
        assert!(err.source().is_none());
        assert_eq!(format!("{}", err), "test");
        assert_eq!(format!("{:?}", err), "BackendError(\"test\")");
    }

    #[test]
    fn from() {
        let err = HoloError::SolveFailed;
        let err: GainError = err.into();
        assert_eq!(format!("{}", err), "Failed to solve linear system");
    }
}
