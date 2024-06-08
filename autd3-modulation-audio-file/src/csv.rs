use autd3_driver::{defined::Freq, derive::*};

use std::{
    fs::File,
    path::{Path, PathBuf},
};

use crate::error::AudioFileError;

#[derive(Modulation, Clone, Builder, PartialEq, Debug)]
pub struct Csv {
    path: PathBuf,
    #[no_change]
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
    #[getset]
    deliminator: u8,
}

impl Csv {
    pub fn new(path: impl AsRef<Path>, sample_rate: Freq<u32>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            config: SamplingConfig::Freq(sample_rate),
            loop_behavior: LoopBehavior::infinite(),
            deliminator: b',',
        }
    }

    fn read_buf(&self) -> Result<Vec<u8>, AudioFileError> {
        let f = File::open(&self.path)?;
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .delimiter(self.deliminator)
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

    #[deprecated(note = "Do not change the sampling configuration", since = "25.0.2")]
    pub fn with_sampling_config(self, config: SamplingConfig) -> Self {
        Self { config, ..self }
    }
}

impl Modulation for Csv {
    fn calc(&self, _: &Geometry) -> ModulationCalcResult {
        Ok(self.read_buf()?)
    }
}

#[cfg(test)]
mod tests {
    use autd3_driver::defined::Hz;

    use crate::tests::create_geometry;

    use super::*;
    use std::io::Write;

    fn create_csv(path: impl AsRef<Path>, data: &[u8]) -> anyhow::Result<()> {
        let mut f = File::create(path)?;
        data.iter().try_for_each(|d| write!(f, "{}\n", d))?;
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    #[case(Ok(vec![0xFF, 0x7F, 0x00]), vec![0xFF, 0x7F, 0x00], 4000 * Hz)]
    fn new(
        #[case] expect: Result<Vec<u8>, AUTDInternalError>,
        #[case] data: Vec<u8>,
        #[case] sample_rate: Freq<u32>,
    ) -> anyhow::Result<()> {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tmp.csv");
        create_csv(&path, &data)?;

        let geometry = create_geometry(1);
        let m = Csv::new(&path, sample_rate);
        assert_eq!(expect, m.calc(&geometry));

        Ok(())
    }

    #[test]
    fn not_exisit() {
        let geometry = create_geometry(1);

        let m = Csv::new("not_exists.csv", 4000 * Hz);
        assert!(m.calc(&geometry).is_err());
    }
}
