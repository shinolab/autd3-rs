use autd3_driver::{error::AUTDInternalError, freq::Freq};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AudioFileError {
    #[error("{0}")]
    Io(std::io::Error),
    #[error("{0}")]
    Wav(hound::Error),
    #[error("RawPCM sampling rate ({0}) must be integer")]
    RawPCMSamplingRateNotInteger(Freq<f64>),
}

// GRCOV_EXCL_START
impl From<std::io::Error> for AudioFileError {
    fn from(e: std::io::Error) -> Self {
        AudioFileError::Io(e)
    }
}

impl From<hound::Error> for AudioFileError {
    fn from(e: hound::Error) -> Self {
        AudioFileError::Wav(e)
    }
}

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
