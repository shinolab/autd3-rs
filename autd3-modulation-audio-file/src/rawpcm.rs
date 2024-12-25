use autd3::modulation::resampler::Resampler;
use autd3_driver::{defined::Freq, derive::*};

use std::{
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

use crate::error::AudioFileError;

/// [`Modulation`] from raw PCM data.
#[derive(Modulation, Debug)]
pub struct RawPCM {
    path: PathBuf,
    #[no_change]
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
    resampler: Option<(Freq<f32>, Box<dyn Resampler>)>,
}

impl RawPCM {
    /// Create a new instance of [`RawPCM`].
    pub fn new<T: TryInto<SamplingConfig>>(
        path: impl AsRef<Path>,
        sampling_config: T,
    ) -> Result<Self, T::Error> {
        Ok(Self {
            path: path.as_ref().to_path_buf(),
            config: sampling_config.try_into()?,
            loop_behavior: LoopBehavior::infinite(),
            resampler: None,
        })
    }

    /// Create a new instance of [`RawPCM`] with resampling.
    ///
    /// # Examples
    ///
    /// ```
    /// use autd3::prelude::*;
    /// use autd3::modulation::resampler::SincInterpolation;
    /// use autd3_modulation_audio_file::RawPCM;
    ///
    /// let path = "path/to/file.dat";
    /// RawPCM::new_with_resample(&path, 2.0 * kHz, 4 * kHz, SincInterpolation::default());
    /// ```
    pub fn new_with_resample<T: TryInto<SamplingConfig>>(
        path: impl AsRef<Path>,
        source: Freq<f32>,
        target: T,
        resampler: impl Resampler + 'static,
    ) -> Result<Self, T::Error> {
        let target = target.try_into()?;
        Ok(Self {
            path: path.as_ref().to_path_buf(),
            config: target,
            loop_behavior: LoopBehavior::infinite(),
            resampler: Some((source, Box::new(resampler))),
        })
    }

    #[tracing::instrument]
    fn read_buf(&self) -> Result<Vec<u8>, AudioFileError> {
        let f = File::open(&self.path)?;
        let mut reader = BufReader::new(f);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;
        tracing::debug!("Read buffer: {:?}", buffer);
        Ok(if let Some((source, resampler)) = &self.resampler {
            resampler.resample(&buffer, *source, self.config)
        } else {
            buffer
        })
    }
}

impl Modulation for RawPCM {
    fn calc(self) -> Result<Vec<u8>, AUTDDriverError> {
        Ok(self.read_buf()?)
    }
}

#[cfg(test)]
mod tests {
    use autd3::{modulation::resampler::SincInterpolation, prelude::kHz};
    use autd3_driver::defined::{Freq, Hz};

    use super::*;
    use std::io::Write;

    fn create_dat(path: impl AsRef<Path>, data: &[u8]) -> anyhow::Result<()> {
        let mut f = File::create(path)?;
        f.write_all(data)?;
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    #[case(Ok(vec![0xFF, 0x7F, 0x00]), vec![0xFF, 0x7F, 0x00], 4000 * Hz)]
    fn new(
        #[case] expect: Result<Vec<u8>, AUTDDriverError>,
        #[case] data: Vec<u8>,
        #[case] sample_rate: Freq<u32>,
    ) -> anyhow::Result<()> {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tmp.dat");
        create_dat(&path, &data)?;

        let m = RawPCM::new(&path, sample_rate)?;
        assert_eq!(expect, m.calc());

        Ok(())
    }

    #[rstest::rstest]
    #[case(vec![127, 217, 255, 217, 127, 37, 0, 37], vec![127, 255, 127, 0], 2.0 * kHz, 4.0 * kHz, SincInterpolation::default())]
    #[case(vec![127, 255, 127, 0], vec![127, 217, 255, 217, 127, 37, 0, 37], 8.0 * kHz, 4.0 * kHz, SincInterpolation::default())]
    #[test]
    fn new_with_resample(
        #[case] expected: Vec<u8>,
        #[case] buffer: Vec<u8>,
        #[case] source: Freq<f32>,
        #[case] target: Freq<f32>,
        #[case] resampler: impl Resampler + 'static,
    ) -> anyhow::Result<()> {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tmp.csv");
        create_dat(&path, &buffer)?;

        let m = RawPCM::new_with_resample(&path, source, target, resampler)?;
        assert_eq!(expected, *m.calc()?);

        Ok(())
    }

    #[test]
    fn not_exisit() -> anyhow::Result<()> {
        let m = RawPCM::new("not_exists.dat", 4000 * Hz)?;
        assert!(m.calc().is_err());
        Ok(())
    }
}
