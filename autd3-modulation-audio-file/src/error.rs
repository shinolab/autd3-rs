use std::num::ParseIntError;

use autd3_core::{firmware::SamplingConfigError, modulation::ModulationError};

#[derive(Debug)]
pub enum AudioFileError {
    Io(std::io::Error),
    Parse(ParseIntError),
    #[cfg(feature = "wav")]
    Wav(hound::Error),
    #[cfg(feature = "csv")]
    Csv(csv::Error),
    SamplingConfig(SamplingConfigError),
    Modulation(ModulationError),
}

// GRCOV_EXCL_START
impl std::fmt::Display for AudioFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioFileError::Io(e) => write!(f, "{}", e),
            AudioFileError::Parse(e) => write!(f, "{}", e),
            #[cfg(feature = "wav")]
            AudioFileError::Wav(e) => write!(f, "{}", e),
            #[cfg(feature = "csv")]
            AudioFileError::Csv(e) => write!(f, "{}", e),
            AudioFileError::SamplingConfig(e) => write!(f, "{}", e),
            AudioFileError::Modulation(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for AudioFileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AudioFileError::Io(e) => Some(e),
            AudioFileError::Parse(e) => Some(e),
            #[cfg(feature = "wav")]
            AudioFileError::Wav(e) => Some(e),
            #[cfg(feature = "csv")]
            AudioFileError::Csv(e) => Some(e),
            AudioFileError::SamplingConfig(e) => Some(e),
            AudioFileError::Modulation(e) => Some(e),
        }
    }
}

impl From<std::io::Error> for AudioFileError {
    fn from(e: std::io::Error) -> Self {
        AudioFileError::Io(e)
    }
}

impl From<ParseIntError> for AudioFileError {
    fn from(e: ParseIntError) -> Self {
        AudioFileError::Parse(e)
    }
}

#[cfg(feature = "wav")]
impl From<hound::Error> for AudioFileError {
    fn from(e: hound::Error) -> Self {
        AudioFileError::Wav(e)
    }
}

#[cfg(feature = "csv")]
impl From<csv::Error> for AudioFileError {
    fn from(e: csv::Error) -> Self {
        AudioFileError::Csv(e)
    }
}

impl From<SamplingConfigError> for AudioFileError {
    fn from(e: SamplingConfigError) -> Self {
        AudioFileError::SamplingConfig(e)
    }
}

impl From<ModulationError> for AudioFileError {
    fn from(e: ModulationError) -> Self {
        AudioFileError::Modulation(e)
    }
}

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
