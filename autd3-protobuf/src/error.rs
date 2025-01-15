use autd3_core::link::LinkError;
use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum AUTDProtoBufError {
    #[error("{0}")]
    Status(#[from] tonic::Status),
    #[error("{0}")]
    DecodeError(#[from] prost::DecodeError),
    #[error("{0}")]
    SendError(String),
    #[error("{0}")]
    TokioSendError(String),
    #[error("{0}")]
    TransportError(#[from] tonic::transport::Error),
    #[cfg(feature = "lightweight")]
    #[error("{0}")]
    HoloError(#[from] autd3_gain_holo::HoloError),
    #[error("{0}")]
    TokioJoinError(String),
    #[error("{0}")]
    AUTDDriverError(#[from] autd3_driver::error::AUTDDriverError),
    #[error("Not supported data")]
    NotSupportedData,
    #[error("Failed to parse data or missing required fields")]
    DataParseError,
    #[cfg(feature = "lightweight")]
    #[error("{0}")]
    UnknownEnumValue(#[from] prost::UnknownEnumValue),
    #[cfg(feature = "lightweight")]
    #[error("{0}")]
    Infallible(#[from] std::convert::Infallible),
}

// GRCOV_EXCL_START

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
        LinkError::new(e.to_string())
    }
}

// GRCOV_EXCL_STOP

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_protobuf_error() {
        let e = AUTDProtoBufError::NotSupportedData;
        assert_eq!(e.to_string(), "Not supported data");
        assert_eq!(format!("{:?}", e), "NotSupportedData");
    }
}
