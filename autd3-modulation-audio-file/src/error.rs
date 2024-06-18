use std::num::ParseIntError;

use autd3_driver::error::AUTDInternalError;
use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum AudioFileError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Parse(#[from] ParseIntError),
    #[error("{0}")]
    Wav(#[from] hound::Error),
    #[error("{0}")]
    Csv(#[from] csv::Error),
}

// GRCOV_EXCL_START
impl From<AudioFileError> for AUTDInternalError {
    fn from(value: AudioFileError) -> Self {
        AUTDInternalError::ModulationError(value.to_string())
    }
}
// GRCOV_EXCL_STOP

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_file_error() {
        let e = AudioFileError::Io(std::io::Error::new(std::io::ErrorKind::Other, "test"));
        assert_eq!(e.to_string(), "test");
        assert_eq!(
            format!("{:?}", e),
            "Io(Custom { kind: Other, error: \"test\" })"
        );
    }
}
