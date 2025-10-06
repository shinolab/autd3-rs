use autd3_core::link::LinkError;

#[derive(Debug)]
#[non_exhaustive]
pub enum AUTDProtoBufError {
    // Do not use `tonic::Status` directly because it cause `clippy::result_large_err`
    // https://github.com/hyperium/tonic/issues/2253
    Status(String),
    SendError(String),
    TransportError(tonic::transport::Error),
}

// GRCOV_EXCL_START

impl std::error::Error for AUTDProtoBufError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AUTDProtoBufError::Status(_) => None,
            AUTDProtoBufError::SendError(_) => None,
            AUTDProtoBufError::TransportError(e) => Some(e),
        }
    }
}

impl std::fmt::Display for AUTDProtoBufError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AUTDProtoBufError::Status(e) => write!(f, "gRPC Status error: {}", e),
            AUTDProtoBufError::SendError(e) => write!(f, "Channel send error: {}", e),
            AUTDProtoBufError::TransportError(e) => write!(f, "Transport error: {}", e),
        }
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

impl From<AUTDProtoBufError> for autd3_core::link::LinkError {
    fn from(e: AUTDProtoBufError) -> Self {
        LinkError::new(e)
    }
}

impl From<tonic::transport::Error> for AUTDProtoBufError {
    fn from(e: tonic::transport::Error) -> Self {
        AUTDProtoBufError::TransportError(e)
    }
}

// GRCOV_EXCL_STOP
