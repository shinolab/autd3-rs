use autd3::modulation::resampler::Resampler;
use autd3_driver::{defined::Freq, derive::*};

use std::{
    fs::File,
    path::{Path, PathBuf},
};

use crate::error::AudioFileError;

#[derive(Modulation, Builder, Debug)]
pub struct Csv {
    path: PathBuf,
    #[get]
    #[set]
    deliminator: u8,
    #[no_change]
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
    resampler: Option<(Freq<f32>, Box<dyn Resampler>)>,
}

impl Csv {
    pub fn new<T: TryInto<SamplingConfig>>(
        path: impl AsRef<Path>,
        sampling_config: T,
    ) -> Result<Self, T::Error> {
        Ok(Self {
            path: path.as_ref().to_path_buf(),
            config: sampling_config.try_into()?,
            loop_behavior: LoopBehavior::infinite(),
            deliminator: b',',
            resampler: None,
        })
    }

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
            deliminator: b',',
            resampler: Some((source, Box::new(resampler))),
        })
    }

    #[tracing::instrument]
    fn read_buf(&self) -> Result<Vec<u8>, AudioFileError> {
        let f = File::open(&self.path)?;
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .delimiter(self.deliminator)
            .from_reader(f);
        let buffer = rdr
            .records()
            .map(|r| {
                let record = r?;
                csv::Result::Ok(
                    record
                        .iter()
                        .map(|x| x.trim().to_owned())
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<csv::Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .map(|s| s.parse::<u8>())
            .collect::<Result<Vec<u8>, _>>()?;
        tracing::debug!("Read buffer: {:?}", buffer);
        Ok(if let Some((source, resampler)) = &self.resampler {
            resampler.resample(&buffer, *source, self.config)
        } else {
            buffer
        })
    }
}

impl Modulation for Csv {
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

    fn create_csv(path: impl AsRef<Path>, data: &[u8]) -> anyhow::Result<()> {
        let mut f = File::create(path)?;
        data.iter().try_for_each(|d| writeln!(f, "{}", d))?;
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    #[case(vec![0xFF, 0x7F, 0x00], 4000 * Hz)]
    fn new(#[case] data: Vec<u8>, #[case] sample_rate: Freq<u32>) -> anyhow::Result<()> {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tmp.csv");
        create_csv(&path, &data)?;

        let m = Csv::new(&path, sample_rate)?;
        assert_eq!(data, *m.calc()?);

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
        create_csv(&path, &buffer)?;

        let m = Csv::new_with_resample(&path, source, target, resampler)?;
        assert_eq!(expected, *m.calc()?);

        Ok(())
    }

    #[test]
    fn not_exisit() -> anyhow::Result<()> {
        let m = Csv::new("not_exists.csv", 4000 * Hz)?;
        assert!(m.calc().is_err());
        Ok(())
    }
}
