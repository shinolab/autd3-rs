use autd3_driver::error::AUTDInternalError;
use thiserror::Error;

/// A interface for error handling in autd3.
#[derive(Error, Debug, PartialEq)]
#[non_exhaustive]
pub enum AUTDError {
    /// Failed to read firmware version.
    #[error("Read firmware info failed: {}", .0.iter().enumerate().filter(|(_, &b)| !b).map(|(i, _)| i.to_string()).collect::<Vec<_>>().join(", "))]
    ReadFirmwareVersionFailed(Vec<bool>),
    /// Failed to read FPGA state.
    #[error("Read FPGA state failed")]
    ReadFPGAStateFailed,
    /// Internal error.
    #[error("{0}")]
    Internal(#[from] AUTDInternalError),
}
