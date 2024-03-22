use autd3_driver::error::AUTDInternalError;
use thiserror::Error;

#[derive(PartialEq)]
pub struct ReadFirmwareInfoState(pub Vec<bool>);

impl std::fmt::Display for ReadFirmwareInfoState {
    // GRCOV_EXCL_START
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Read firmware info failed: {}",
            self.0
                .iter()
                .enumerate()
                .filter(|(_, &b)| !b)
                .map(|(i, _)| i.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
    // GRCOV_EXCL_STOP
}

impl std::fmt::Debug for ReadFirmwareInfoState {
    // GRCOV_EXCL_START
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as std::fmt::Display>::fmt(self, f)
    }
    // GRCOV_EXCL_STOP
}

#[derive(Error, Debug, PartialEq)]
pub enum AUTDError {
    #[error("Device id ({0}) is specified, but only {1} AUTDs are connected.")]
    GroupedOutOfRange(usize, usize),
    #[error("{0}")]
    ReadFirmwareInfoFailed(ReadFirmwareInfoState),
    #[error("Read FPGA state failed")]
    ReadFPGAStateFailed,
    #[error("{0}")]
    Internal(AUTDInternalError),
}

impl From<AUTDInternalError> for AUTDError {
    // GRCOV_EXCL_START
    fn from(e: AUTDInternalError) -> Self {
        AUTDError::Internal(e)
    }
    // GRCOV_EXCL_STOP
}
