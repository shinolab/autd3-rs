use autd3_driver::error::AUTDDriverError;
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

impl From<HoloError> for AUTDDriverError {
    fn from(value: HoloError) -> Self {
        AUTDDriverError::GainError(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    #[cfg_attr(miri, ignore)]
    fn solve_failed() {
        let err = HoloError::SolveFailed;
        assert!(err.source().is_none());
        assert_eq!(format!("{}", err), "Failed to solve linear system");
        assert_eq!(format!("{:?}", err), "SolveFailed");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn backend_error() {
        let err = HoloError::BackendError("test".to_string());
        assert!(err.source().is_none());
        assert_eq!(format!("{}", err), "test");
        assert_eq!(format!("{:?}", err), "BackendError(\"test\")");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn from() {
        let err = HoloError::SolveFailed;
        let err: AUTDDriverError = err.into();
        assert_eq!(format!("{}", err), "Failed to solve linear system");
    }
}
