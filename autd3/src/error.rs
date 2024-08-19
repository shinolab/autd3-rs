use autd3_driver::error::AUTDInternalError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
#[non_exhaustive]
pub enum AUTDError {
    #[error("Read firmware info failed: {}", .0.iter().enumerate().filter(|(_, &b)| !b).map(|(i, _)| i.to_string()).collect::<Vec<_>>().join(", "))]
    ReadFirmwareVersionFailed(Vec<bool>),
    #[error("Read FPGA state failed")]
    ReadFPGAStateFailed,
    #[error("{0}")]
    Internal(#[from] AUTDInternalError),
}
