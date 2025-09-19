use autd3_core::derive::*;

use std::{fmt::Debug, fs::File, path::Path};

use crate::error::AudioFileError;

/// The option of [`Csv`].
#[derive(Debug, Clone)]
pub struct CsvOption {
    /// The delimiter of CSV file.
    pub delimiter: u8,
    /// Whether the CSV file has headers.
    pub has_headers: bool,
}

impl Default for CsvOption {
    fn default() -> Self {
        Self {
            delimiter: b',',
            has_headers: false,
        }
    }
}

/// [`Modulation`] from CSV data.
#[derive(Modulation, Debug, Clone)]
pub struct Csv {
    sampling_config: SamplingConfig,
    buffer: Vec<u8>,
}

impl Csv {
    /// Create a new [`Csv`].
    pub fn new<P, Config>(
        path: P,
        sampling_config: Config,
        option: CsvOption,
    ) -> Result<Self, AudioFileError>
    where
        P: AsRef<Path> + Clone + Debug,
        Config: Into<SamplingConfig> + Debug + Copy,
    {
        let f = File::open(&path)?;
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(option.has_headers)
            .delimiter(option.delimiter)
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
        Ok(Self {
            sampling_config: sampling_config.into(),
            buffer,
        })
    }

    /// Write a [`Modulation`] into a writer as CSV format.
    pub fn write<Writer: std::io::Write, M: Modulation>(
        m: M,
        writer: Writer,
        option: CsvOption,
    ) -> Result<(), AudioFileError> {
        let sample_rate = m.sampling_config().freq()?.hz();
        let buffer = m.calc(&FirmwareLimits {
            mod_buf_size_max: u32::MAX,
            ..FirmwareLimits::unused()
        })?;
        let mut writer = csv::WriterBuilder::new()
            .delimiter(option.delimiter)
            .from_writer(writer);
        if option.has_headers {
            writer.write_record(&[format!("Buffer (sampling rate = {sample_rate} Hz)")])?;
        }
        buffer
            .into_iter()
            .try_for_each(|b| writer.write_record(&[b.to_string()]))?;
        Ok(())
    }
}

impl Modulation for Csv {
    fn calc(self, _: &FirmwareLimits) -> Result<Vec<u8>, ModulationError> {
        Ok(self.buffer)
    }

    fn sampling_config(&self) -> SamplingConfig {
        self.sampling_config
    }
}

#[cfg(test)]
mod tests {
    use autd3_core::common::{Freq, Hz};

    use super::*;
    use std::io::Write;

    fn create_csv(path: impl AsRef<Path>, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        let mut f = File::create(path)?;
        data.iter().try_for_each(|d| writeln!(f, "{d}"))?;
        Ok(())
    }

    #[rstest::rstest]
    #[case(vec![0xFF, 0x7F, 0x00], 4000. * Hz)]
    fn new(
        #[case] data: Vec<u8>,
        #[case] sample_rate: Freq<f32>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tmp.csv");
        create_csv(&path, &data)?;

        let m = Csv::new(path, sample_rate, CsvOption::default())?;
        assert_eq!(sample_rate.hz(), m.sampling_config().freq()?.hz());
        assert_eq!(data, *m.calc(&FirmwareLimits::unused())?);

        Ok(())
    }

    #[rstest::rstest]
    #[case("Buffer (sampling rate = 4000 Hz)\n0\n128\n255\n", true)]
    #[case("0\n128\n255\n", false)]
    fn write(
        #[case] expect: &str,
        #[case] has_headers: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        #[derive(Clone)]
        struct TestMod {
            data: Vec<u8>,
            rate: f32,
        }
        impl Modulation for TestMod {
            fn calc(self, _: &FirmwareLimits) -> Result<Vec<u8>, ModulationError> {
                Ok(self.data)
            }
            fn sampling_config(&self) -> SamplingConfig {
                SamplingConfig::new(self.rate * Hz)
            }
        }

        let m = TestMod {
            data: vec![0u8, 128u8, 255u8],
            rate: 4000.0,
        };
        let mut wtr = Vec::new();
        Csv::write(
            m,
            &mut wtr,
            CsvOption {
                delimiter: b',',
                has_headers,
            },
        )?;

        assert_eq!(expect, String::from_utf8(wtr)?);

        Ok(())
    }

    #[test]
    fn not_exists() -> Result<(), Box<dyn std::error::Error>> {
        assert!(
            Csv::new(
                Path::new("not_exists.csv"),
                4000. * Hz,
                CsvOption::default(),
            )
            .is_err()
        );
        Ok(())
    }
}
