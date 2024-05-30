use autd3_driver::error::AUTDInternalError;
use thiserror::Error;

#[derive(PartialEq)]
pub struct ReadFirmwareVersionState(pub Vec<bool>);

impl std::fmt::Display for ReadFirmwareVersionState {
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
}

impl std::fmt::Debug for ReadFirmwareVersionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as std::fmt::Display>::fmt(self, f)
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum AUTDError {
    #[error("{0}")]
    ReadFirmwareVersionFailed(ReadFirmwareVersionState),
    #[error("Read FPGA state failed")]
    ReadFPGAStateFailed,
    #[error("{0}")]
    Internal(#[from] AUTDInternalError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_firmware_version_state_fmt() {
        let state = ReadFirmwareVersionState(vec![true, false, true]);
        assert_eq!(
            format!("{}", state),
            "Read firmware info failed: 1".to_string()
        );
        assert_eq!(
            format!("{:?}", state),
            "Read firmware info failed: 1".to_string()
        );
    }
}
