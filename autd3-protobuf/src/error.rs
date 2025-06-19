use autd3_core::link::LinkError;
use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum AUTDProtoBufError {
    // Do not use `tonic::Status` directly because it cause `clippy::result_large_err`
    // https://github.com/hyperium/tonic/issues/2253
    #[error("{0}")]
    Status(String),
    #[error("{0}")]
    SendError(String),
    #[error("{0}")]
    TransportError(#[from] tonic::transport::Error),
}

// GRCOV_EXCL_START

impl From<tonic::Status> for AUTDProtoBufError {
    fn from(e: tonic::Status) -> Self {
        AUTDProtoBufError::Status(e.to_string())
    }
}

impl From<AUTDProtoBufError> for tonic::Status {
    fn from(e: AUTDProtoBufError) -> Self {
        tonic::Status::internal(e.to_string())
    }
}

impl<T> From<std::sync::mpsc::SendError<T>> for AUTDProtoBufError {
    fn from(e: std::sync::mpsc::SendError<T>) -> Self {
        AUTDProtoBufError::SendError(e.to_string())
    }
}

impl From<AUTDProtoBufError> for autd3_core::link::LinkError {
    fn from(e: AUTDProtoBufError) -> Self {
        LinkError::new(e)
    }
}

// GRCOV_EXCL_STOP
