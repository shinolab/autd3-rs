use autd3_driver::{defined::Freq, derive::*, utils::float::is_integer};

use std::{
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

use crate::error::AudioFileError;

#[derive(Modulation, Clone, PartialEq, Debug)]
pub struct RawPCM {
    sample_rate: Freq<u32>,
    path: PathBuf,
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
}

impl RawPCM {
    pub fn new(path: impl AsRef<Path>, sample_rate: Freq<u32>) -> Self {
        Self {
            sample_rate,
            path: path.as_ref().to_path_buf(),
            config: SamplingConfig::Division(5120),
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
    fn calc(&self, geometry: &Geometry) -> ModulationCalcResult {
        let new_rate = self.sampling_config().freq(geometry.ultrasound_freq())?;
        if !is_integer(new_rate.hz()) {
            return Err(AudioFileError::RawPCMSamplingRateNotInteger(new_rate).into());
        }
        let buf = wav_io::resample::linear(
            self.read_buf()?,
            1,
            self.sample_rate.hz(),
            new_rate.hz() as u32,
        )
        .iter()
        .map(|&d| d.round() as u8)
        .collect::<Vec<_>>();
        Ok(buf.clone())
    }
}

#[cfg(test)]
mod tests {
    use autd3_driver::defined::{kHz, Hz};

    use crate::tests::create_geometry;

    use super::*;
    use std::io::Write;

    fn create_dat(path: impl AsRef<Path>, data: &[u8]) -> anyhow::Result<()> {
        let mut f = File::create(path)?;
        f.write_all(data)?;
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    #[case(Ok(vec![0xFF, 0x7F, 0x00]), vec![0xFF, 0x7F, 0x00], 4000 * Hz, SamplingConfig::Division(5120))]
    #[case(Err(AudioFileError::RawPCMSamplingRateNotInteger(SamplingConfig::FreqNearest(10.5*kHz).freq(40*kHz).unwrap()).into()), vec![0xFF, 0x7F, 0x00], 4000 * Hz, SamplingConfig::FreqNearest(10.5*kHz))]
    fn new(
        #[case] expect: Result<Vec<u8>, AUTDInternalError>,
        #[case] data: Vec<u8>,
        #[case] sample_rate: Freq<u32>,
        #[case] config: SamplingConfig,
    ) -> anyhow::Result<()> {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tmp.dat");
        create_dat(&path, &data)?;

        let geometry = create_geometry(1);
        let m = RawPCM::new(&path, sample_rate).with_sampling_config(config);
        assert_eq!(expect, m.calc(&geometry));

        Ok(())
    }

    #[test]
    fn not_exisit() {
        let geometry = create_geometry(1);

        let m = RawPCM::new("not_exists.dat", 4000 * Hz);
        assert!(m.calc(&geometry).is_err());
    }
}
