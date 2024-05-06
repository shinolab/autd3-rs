use autd3_driver::derive::*;

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
#[derive(Modulation, Clone, PartialEq, Debug)]
pub struct RawPCM {
    sample_rate: u32,
    path: PathBuf,
    config: SamplingConfiguration,
    loop_behavior: LoopBehavior,
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
            loop_behavior: LoopBehavior::infinite(),
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
    fn calc(&self, geometry: &Geometry) -> Result<HashMap<usize, Vec<u8>>, AUTDInternalError> {
        Self::transform(geometry, |dev| {
            Ok(wav_io::resample::linear(
                self.read_buf()?,
                1,
                self.sample_rate,
                self.sampling_config().freq(dev.ultrasound_freq())? as u32,
            )
            .iter()
            .map(|&d| d.round() as u8)
            .collect())
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::create_geometry;

    use super::*;
    use std::io::Write;

    fn create_dat(path: impl AsRef<Path>, data: &[u8]) -> anyhow::Result<()> {
        let mut f = File::create(path)?;
        f.write_all(data)?;
        Ok(())
    }

    #[test]
    fn test_rawpcm_new() -> anyhow::Result<()> {
        let geometry = create_geometry(1);
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("tmp.dat");
        create_dat(&path, &[0xFF, 0x7F, 0x00])?;
        let m = RawPCM::new(&path, 4000);
        assert_eq!(
            m.calc(&geometry)?,
            HashMap::from([(0, vec![0xFF, 0x7F, 0x00])])
        );

        let m = RawPCM::new("not_exists.dat", 4000);
        assert!(m.calc(&geometry).is_err());

        Ok(())
    }

    #[test]
    fn test_rawpcm_clone() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("tmp.dat");
        create_dat(&path, &[0xFF, 0xFF])?;
        let m = RawPCM::new(&path, 4000);
        assert_eq!(m, m.clone());
        Ok(())
    }
}
