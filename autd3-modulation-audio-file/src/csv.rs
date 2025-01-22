use autd3_core::{defined::Freq, derive::*, resampler::Resampler};

use std::{fmt::Debug, fs::File, path::Path, rc::Rc};

use crate::error::AudioFileError;

/// The option of [`Csv`].
#[derive(Debug, Clone)]
pub struct CsvOption {
    /// The deliminator of CSV file.
    pub deliminator: u8,
    resampler: Option<(Freq<f32>, Rc<dyn Resampler>)>,
}

impl Default for CsvOption {
    fn default() -> Self {
        Self {
            deliminator: b',',
            resampler: None,
        }
    }
}

/// [`Modulation`] from CSV data.
#[derive(Modulation, Debug)]
pub struct Csv<'a, Config, E>
where
    E: Debug,
    SamplingConfigError: From<E>,
    Config: TryInto<SamplingConfig, Error = E> + Debug + Copy,
{
    /// The path to the CSV file.
    pub path: &'a Path,
    /// The sampling configuration of the CSV file.
    pub sampling_config: Config,
    /// The option of [`Csv`].
    pub option: CsvOption,
}

impl<'a> Csv<'a, Freq<f32>, SamplingConfigError> {
    /// Resample the csv data to the target frequency.
    ///
    /// # Examples
    ///
    /// ```
    /// use autd3_core::{resampler::SincInterpolation, defined::kHz};
    /// use autd3_modulation_audio_file::Csv;
    ///
    /// let path = "path/to/file.csv";
    /// Csv {
    ///     path: std::path::Path::new(path),
    ///     sampling_config: 2.0 * kHz,
    ///     option: Default::default(),
    /// }.with_resample(4 * kHz, SincInterpolation::default());
    /// ```
    pub fn with_resample<T, E>(
        self,
        target: T,
        resampler: impl Resampler + 'static,
    ) -> Csv<'a, T, E>
    where
        E: Debug,
        SamplingConfigError: From<E>,
        T: TryInto<SamplingConfig, Error = E> + Debug + Copy,
    {
        let source = self.sampling_config;
        Csv {
            path: self.path,
            sampling_config: target,
            option: CsvOption {
                resampler: Some((source, Rc::new(resampler))),
                ..self.option
            },
        }
    }
}

impl<Config, E> Csv<'_, Config, E>
where
    E: Debug,
    SamplingConfigError: From<E>,
    Config: TryInto<SamplingConfig, Error = E> + Debug + Copy,
{
    #[tracing::instrument]
    fn read_buf(&self) -> Result<Vec<u8>, AudioFileError> {
        let f = File::open(self.path)?;
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .delimiter(self.option.deliminator)
            .from_reader(f);
        Ok(rdr
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
            .collect::<Result<Vec<u8>, _>>()?)
    }
}

impl<Config, E> Modulation for Csv<'_, Config, E>
where
    E: Debug,
    SamplingConfigError: From<E>,
    Config: TryInto<SamplingConfig, Error = E> + Debug + Copy,
{
    fn calc(self) -> Result<Vec<u8>, ModulationError> {
        let buffer = self.read_buf()?;
        tracing::debug!("Read buffer: {:?}", buffer);
        let target = self.sampling_config()?;
        Ok(if let Some((source, resampler)) = self.option.resampler {
            resampler.resample(&buffer, source, target)
        } else {
            buffer
        })
    }

    fn sampling_config(&self) -> Result<SamplingConfig, ModulationError> {
        Ok(self
            .sampling_config
            .try_into()
            .map_err(SamplingConfigError::from)?)
    }
}

#[cfg(test)]
mod tests {
    use autd3_core::{
        defined::{kHz, Freq, Hz},
        resampler::SincInterpolation,
    };

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

        let m = Csv {
            path: path.as_path(),
            sampling_config: sample_rate,
            option: CsvOption::default(),
        };
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

        let m = Csv {
            path: path.as_path(),
            sampling_config: source,
            option: CsvOption::default(),
        }
        .with_resample(target, resampler);

        assert_eq!(expected, *m.calc()?);

        Ok(())
    }

    #[test]
    fn not_exisit() -> anyhow::Result<()> {
        let m = Csv {
            path: Path::new("not_exists.csv"),
            sampling_config: 4000 * Hz,
            option: CsvOption::default(),
        };
        assert!(m.calc().is_err());
        Ok(())
    }
}
