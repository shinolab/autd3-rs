use autd3_driver::{common::EmitIntensity, derive::*};

use std::{
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

use crate::error::AudioFileError;

/// Modulation constructed from a raw PCM data
///
/// The raw PCM data must be 8bit unsigned integer.
///
/// The raw PCM data is resampled to the sampling frequency of Modulation.
#[derive(Modulation, Clone)]
pub struct RawPCM {
    sample_rate: u32,
    path: PathBuf,
    config: SamplingConfiguration,
}

impl RawPCM {
    /// Constructor
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the raw PCM file
    /// * `sample_rate` - Sampling frequency of the raw PCM file
    ///
    pub fn new(path: impl AsRef<Path>, sample_rate: u32) -> Self {
        Self {
            sample_rate,
            path: path.as_ref().to_path_buf(),
            config: SamplingConfiguration::FREQ_4K_HZ,
        }
    }

    fn read_buf(&self) -> Result<Vec<f32>, AudioFileError> {
        let f = File::open(&self.path)?;
        let mut reader = BufReader::new(f);
        let mut raw_buffer = Vec::new();
        reader.read_to_end(&mut raw_buffer)?;
        Ok(raw_buffer.into_iter().map(f32::from).collect())
    }
}

impl Modulation for RawPCM {
    fn calc(&self) -> Result<Vec<EmitIntensity>, AUTDInternalError> {
        Ok(wav_io::resample::linear(
            self.read_buf()?,
            1,
            self.sample_rate,
            self.sampling_config().frequency() as u32,
        )
        .iter()
        .map(|&d| EmitIntensity::new(d.round() as u8))
        .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn create_dat(path: impl AsRef<Path>, data: &[u8]) {
        std::fs::create_dir_all(path.as_ref().parent().unwrap()).unwrap();
        if path.as_ref().exists() {
            std::fs::remove_file(path.as_ref()).unwrap();
        }
        let mut f = File::create(path).unwrap();
        f.write_all(data).unwrap();
    }

    #[test]
    fn test_rawpcm_new() {
        let home_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let path = Path::new(&home_dir).join("tmp").join("tmp.dat");
        create_dat(&path, &[0xFF, 0x7F, 0x00]);
        let m = RawPCM::new(&path, 4000);
        assert_eq!(
            m.calc().unwrap(),
            vec![
                EmitIntensity::new(0xFF),
                EmitIntensity::new(0x7F),
                EmitIntensity::new(0x00)
            ]
        );

        let m = RawPCM::new("not_exists.dat", 4000);
        assert!(m.calc().is_err());
    }

    #[test]
    fn test_rawpcm_clone() {
        let home_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let path = Path::new(&home_dir).join("tmp").join("tmp2.dat");
        create_dat(&path, &[0xFF, 0xFF]);
        let m = RawPCM::new(Path::new(&home_dir).join("tmp").join("tmp2.dat"), 4000);
        let m2 = m.clone();
        assert_eq!(m.sampling_config(), m2.sampling_config());
    }
}
