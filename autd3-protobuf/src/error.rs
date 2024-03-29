use std::error::Error;

use thiserror::Error;
use tonic::Status;

#[derive(Error, Debug)]
pub enum AUTDProtoBufError {
    #[error("{0}")]
    Status(String),
    #[error("{0}")]
    SendError(String),
    #[error("{0}")]
    TokioSendError(String),
    #[error("{0}")]
    TransportError(String),
    #[error("{0}")]
    TokioJoinError(String),
    #[error("{0}")]
    AUTDInternalError(autd3_driver::error::AUTDInternalError),
    #[error("This data is not supported.")]
    NotSupportedData,
    #[error("Failed to parse data.")]
    DataParseError,
}

// GRCOV_EXCL_START
impl From<autd3_driver::error::AUTDInternalError> for AUTDProtoBufError {
    fn from(e: autd3_driver::error::AUTDInternalError) -> Self {
        AUTDProtoBufError::AUTDInternalError(e)
    }
}

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

impl From<tonic::transport::Error> for AUTDProtoBufError {
    fn from(e: tonic::transport::Error) -> Self {
        match e.source() {
            Some(source) => AUTDProtoBufError::TransportError(source.to_string()),
            None => AUTDProtoBufError::TransportError(e.to_string()),
        }
    }
}

impl From<AUTDProtoBufError> for autd3_driver::error::AUTDInternalError {
    fn from(e: AUTDProtoBufError) -> Self {
        autd3_driver::error::AUTDInternalError::LinkError(e.to_string())
    }
}

#[cfg(feature = "lightweight")]
impl From<AUTDProtoBufError> for autd3::error::AUTDError {
    fn from(e: AUTDProtoBufError) -> Self {
        Self::Internal(e.into())
    }
}

pub fn match_for_io_error(err_status: &Status) -> Option<&std::io::Error> {
    let mut err: &(dyn Error + 'static) = err_status;
    loop {
        if let Some(io_err) = err.downcast_ref::<std::io::Error>() {
            return Some(io_err);
        }
        if let Some(h2_err) = err.downcast_ref::<h2::Error>() {
            if let Some(io_err) = h2_err.get_io() {
                return Some(io_err);
            }
        }
        err = match err.source() {
            Some(err) => err,
            None => return None,
        };
    }
}
// GRCOV_EXCL_STOP

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protobuf_error() {
        let e = AUTDProtoBufError::NotSupportedData;
        assert_eq!(e.to_string(), "This data is not supported.");
        assert_eq!(format!("{:?}", e), "NotSupportedData");
    }
}
