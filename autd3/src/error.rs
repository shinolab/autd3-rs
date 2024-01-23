use autd3_driver::error::AUTDInternalError;
use thiserror::Error;

#[derive(PartialEq)]
pub struct ReadFirmwareInfoState(pub Vec<bool>);

impl std::fmt::Display for ReadFirmwareInfoState {
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Read firmware info failed: {}",
            self.0
                .iter()
                .enumerate()
                .filter_map(|(i, b)| if *b { None } else { Some(i.to_string()) })
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl std::fmt::Debug for ReadFirmwareInfoState {
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as std::fmt::Display>::fmt(self, f)
    }
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
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn from(e: AUTDInternalError) -> Self {
        AUTDError::Internal(e)
    }
}
