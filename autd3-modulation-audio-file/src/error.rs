use autd3_driver::error::AUTDInternalError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AudioFileError {
    #[error("{0}")]
    Io(std::io::Error),
    #[error("{0}")]
    Wav(hound::Error),
}

impl From<std::io::Error> for AudioFileError {
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn from(e: std::io::Error) -> Self {
        AudioFileError::Io(e)
    }
}

impl From<hound::Error> for AudioFileError {
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn from(e: hound::Error) -> Self {
        AudioFileError::Wav(e)
    }
}

impl From<AudioFileError> for AUTDInternalError {
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn from(value: AudioFileError) -> Self {
        AUTDInternalError::ModulationError(value.to_string())
    }
}

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
