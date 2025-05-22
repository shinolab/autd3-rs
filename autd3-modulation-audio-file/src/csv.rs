use autd3_core::derive::*;

use std::{fmt::Debug, fs::File, path::Path};

use crate::error::AudioFileError;

/// The option of [`Csv`].
#[derive(Debug, Clone)]
pub struct CsvOption {
    /// The delimiter of CSV file.
    pub delimiter: u8,
}

impl Default for CsvOption {
    fn default() -> Self {
        Self { delimiter: b',' }
    }
}

/// [`Modulation`] from CSV data.
#[derive(Modulation, Debug, Clone)]
pub struct Csv<P, Config>
where
    P: AsRef<Path> + Clone + Debug,
    Config: Into<SamplingConfig> + Debug + Copy,
{
    /// The path to the CSV file.
    pub path: P,
    /// The sampling configuration of the CSV file.
    pub sampling_config: Config,
    /// The option of [`Csv`].
    pub option: CsvOption,
}

impl<P, Config> Csv<P, Config>
where
    P: AsRef<Path> + Clone + Debug,
    Config: Into<SamplingConfig> + Debug + Copy,
{
    /// Create a new [`Csv`].
    #[must_use]
    pub const fn new(path: P, sampling_config: Config, option: CsvOption) -> Self {
        Self {
            path,
            sampling_config,
            option,
        }
    }

    #[tracing::instrument]
    fn read_buf(&self) -> Result<Vec<u8>, AudioFileError> {
        let f = File::open(&self.path)?;
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .delimiter(self.option.delimiter)
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

impl<P, Config> Modulation for Csv<P, Config>
where
    P: AsRef<Path> + Clone + Debug,
    Config: Into<SamplingConfig> + Debug + Copy,
{
    fn calc(self) -> Result<Vec<u8>, ModulationError> {
        let buffer = self.read_buf()?;
        tracing::debug!("Read buffer: {:?}", buffer);
        Ok(buffer)
    }

    fn sampling_config(&self) -> SamplingConfig {
        self.sampling_config.into()
    }
}

#[cfg(test)]
mod tests {
    use autd3_core::common::{Freq, Hz};

    use super::*;
    use std::io::Write;

    fn create_csv(path: impl AsRef<Path>, data: &[u8]) -> anyhow::Result<()> {
        let mut f = File::create(path)?;
        data.iter().try_for_each(|d| writeln!(f, "{}", d))?;
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    #[case(vec![0xFF, 0x7F, 0x00], 4000. * Hz)]
    fn new(#[case] data: Vec<u8>, #[case] sample_rate: Freq<f32>) -> anyhow::Result<()> {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tmp.csv");
        create_csv(&path, &data)?;

        let m = Csv::new(path, sample_rate, CsvOption::default());
        assert_eq!(sample_rate.hz(), m.sampling_config().freq()?.hz());
        assert_eq!(data, *m.calc()?);

        Ok(())
    }

    #[test]
    fn not_exists() -> anyhow::Result<()> {
        let m = Csv {
            path: Path::new("not_exists.csv"),
            sampling_config: 4000. * Hz,
            option: CsvOption::default(),
        };
        assert!(m.calc().is_err());
        Ok(())
    }
}
