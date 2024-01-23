use autd3_driver::{common::EmitIntensity, derive::*};

use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
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
    raw_buffer: Vec<f32>,
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
    pub fn new<P: AsRef<Path>>(path: P, sample_rate: u32) -> Result<Self, AudioFileError> {
        let f = File::open(path)?;
        let mut reader = BufReader::new(f);
        let mut raw_buffer = Vec::new();
        reader.read_to_end(&mut raw_buffer)?;
        Ok(Self {
            sample_rate,
            raw_buffer: raw_buffer.iter().map(|&v| v as f32 / 255.).collect(),
            config: SamplingConfiguration::FREQ_4K_HZ,
        })
    }
}

impl Modulation for RawPCM {
    fn calc(&self) -> Result<Vec<EmitIntensity>, AUTDInternalError> {
        Ok(wav_io::resample::linear(
            self.raw_buffer.clone(),
            1,
            self.sample_rate,
            self.sampling_config().frequency() as u32,
        )
        .iter()
        .map(|&d| EmitIntensity::new((d * 255.).round() as u8))
        .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn create_dat<P: AsRef<Path>>(path: P, data: &[u8]) {
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
        assert!(m.is_ok());
        let m = m.unwrap();
        assert_eq!(
            m.calc().unwrap(),
            vec![
                EmitIntensity::new(0xFF),
                EmitIntensity::new(0x7F),
                EmitIntensity::new(0x00)
            ]
        );

        let m = RawPCM::new("not_exists.dat", 4000);
        assert!(m.is_err());
    }

    #[test]
    fn test_rawpcm_clone() {
        let home_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let path = Path::new(&home_dir).join("tmp").join("tmp2.dat");
        create_dat(&path, &[0xFF, 0xFF]);
        let m = RawPCM::new(Path::new(&home_dir).join("tmp").join("tmp2.dat"), 4000).unwrap();
        let m2 = m.clone();
        assert_eq!(m.sampling_config(), m2.sampling_config());
    }
}
