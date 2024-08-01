use autd3_driver::derive::*;

use std::{
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

use crate::error::AudioFileError;

#[derive(Modulation, Clone, PartialEq, Debug)]
pub struct RawPCM {
    path: PathBuf,
    #[no_change]
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
}

impl RawPCM {
    pub fn new<T: TryInto<SamplingConfig>>(
        path: impl AsRef<Path>,
        sampling_config: T,
    ) -> Result<Self, T::Error> {
        Ok(Self {
            path: path.as_ref().to_path_buf(),
            config: sampling_config.try_into()?,
            loop_behavior: LoopBehavior::infinite(),
        })
    }

    fn read_buf(&self) -> Result<Vec<u8>, AudioFileError> {
        let f = File::open(&self.path)?;
        let mut reader = BufReader::new(f);
        let mut raw_buffer = Vec::new();
        reader.read_to_end(&mut raw_buffer)?;
        Ok(raw_buffer)
    }
}

impl Modulation for RawPCM {
    fn calc(&self) -> ModulationCalcResult {
        Ok(Arc::new(self.read_buf()?))
    }

    #[tracing::instrument(level = "debug", skip(_geometry))]
    // GRCOV_EXCL_START
    fn trace(&self, _geometry: &Geometry) {
        tracing::debug!("{}", tynm::type_name::<Self>());
    }
    // GRCOV_EXCL_STOP
}

#[cfg(test)]
mod tests {
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
    #[case(Ok(Arc::new(vec![0xFF, 0x7F, 0x00])), vec![0xFF, 0x7F, 0x00], 4000 * Hz)]
    fn new(
        #[case] expect: ModulationCalcResult,
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

    #[test]
    fn not_exisit() -> anyhow::Result<()> {
        let m = RawPCM::new("not_exists.dat", 4000 * Hz)?;
        assert!(m.calc().is_err());
        Ok(())
    }
}
