use std::num::ParseIntError;

use autd3_core::{firmware::SamplingConfigError, modulation::ModulationError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AudioFileError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Parse(#[from] ParseIntError),
    #[cfg(feature = "wav")]
    #[error("{0}")]
    Wav(#[from] hound::Error),
    #[cfg(feature = "csv")]
    #[error("{0}")]
    Csv(#[from] csv::Error),
    #[error("{0}")]
    SamplingConfig(#[from] SamplingConfigError),
}

// GRCOV_EXCL_START
impl From<AudioFileError> for ModulationError {
    fn from(value: AudioFileError) -> Self {
        ModulationError::new(value)
    }
}
// GRCOV_EXCL_STOP

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_file_error() {
        let e = AudioFileError::Io(std::io::Error::other("test"));
        assert_eq!(e.to_string(), "test");
        assert_eq!(
            format!("{e:?}"),
            "Io(Custom { kind: Other, error: \"test\" })"
        );
    }
}
