use autd3_driver::error::AUTDInternalError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HoloError {
    #[error("Failed to solve linear system")]
    SolveFailed,
    #[error("{0}")]
    BackendError(String),
    #[error("{0}")]
    BackendCreationError(String),
    #[error("Invalid operation")]
    InvalidOperation,
    #[error("Failed to compute SVD")]
    SVDFailed,
}

impl From<HoloError> for AUTDInternalError {
    fn from(value: HoloError) -> Self {
        AUTDInternalError::GainError(value.to_string())
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
    fn backend_creation_error() {
        let err = HoloError::BackendCreationError("test".to_string());
        assert!(err.source().is_none());
        assert_eq!(format!("{}", err), "test");
        assert_eq!(format!("{:?}", err), "BackendCreationError(\"test\")");
    }

    #[test]
    fn from() {
        let err = HoloError::SolveFailed;
        let err: AUTDInternalError = err.into();
        assert_eq!(format!("{}", err), "Failed to solve linear system");
    }
}
