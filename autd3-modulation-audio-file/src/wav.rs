use autd3_driver::derive::*;
use hound::SampleFormat;

use std::path::{Path, PathBuf};

use crate::error::AudioFileError;

#[derive(Modulation, Clone, PartialEq, Debug)]
pub struct Wav {
    path: PathBuf,
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
}

impl Wav {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            config: SamplingConfig::Division(5120),
            loop_behavior: LoopBehavior::infinite(),
        }
    }

    fn read_buf(&self) -> Result<(Vec<f32>, u32), AudioFileError> {
        let mut reader = hound::WavReader::open(&self.path)?;
        let spec = reader.spec();
        if spec.channels != 1 {
            return Err(AudioFileError::Wav(hound::Error::Unsupported));
        }
        let buf = match spec.sample_format {
            SampleFormat::Int => {
                let raw_buffer = reader.samples::<i32>().collect::<Result<Vec<_>, _>>()?;
                match spec.bits_per_sample {
                    8 => raw_buffer
                        .iter()
                        .map(|i| (i - i8::MIN as i32) as f32)
                        .collect(),
                    16 => raw_buffer
                        .iter()
                        .map(|i| (i - i16::MIN as i32) as f32 / 257.)
                        .collect(),
                    24 => raw_buffer
                        .iter()
                        .map(|i| (i + 8388608i32) as f32 / 65793.)
                        .collect(),
                    32 => raw_buffer
                        .iter()
                        .map(|&i| (i as i64 - i32::MIN as i64) as f32 / 16843009.)
                        .collect(),
                    _ => return Err(AudioFileError::Wav(hound::Error::Unsupported)), // GRCOV_EXCL_LINE
                }
            }
            SampleFormat::Float => {
                let raw_buffer = reader.samples::<f32>().collect::<Result<Vec<_>, _>>()?;
                match spec.bits_per_sample {
                    32 => raw_buffer.iter().map(|&i| (i + 1.0) / 2. * 255.).collect(),
                    _ => return Err(AudioFileError::Wav(hound::Error::Unsupported)), // GRCOV_EXCL_LINE
                }
            }
        };

        Ok((buf, spec.sample_rate))
    }
}

impl Modulation for Wav {
    #[allow(clippy::unnecessary_cast)]
    fn calc(&self, geometry: &Geometry) -> ModulationCalcResult {
        let (raw_buffer, sample_rate) = self.read_buf()?;
        let buf = wav_io::resample::linear(
            raw_buffer.clone(),
            1,
            sample_rate,
            self.sampling_config()
                .freq(geometry.ultrasound_freq())?
                .hz() as u32,
        )
        .iter()
        .map(|&d| d.round() as u8)
        .collect::<Vec<_>>();
        Ok(buf)
    }

    #[tracing::instrument(level = "debug", skip(_geometry))]
    // GRCOV_EXCL_START
    fn trace(&self, _geometry: &Geometry) {
        tracing::info!("{}", tynm::type_name::<Self>());
    }
    // GRCOV_EXCL_STOP
}

#[cfg(test)]
mod tests {
    use crate::tests::create_geometry;

    use super::*;

    fn create_wav(
        path: impl AsRef<Path>,
        spec: hound::WavSpec,
        data: &[impl hound::Sample + Copy],
    ) -> anyhow::Result<()> {
        let mut writer = hound::WavWriter::create(path, spec)?;
        data.iter().try_for_each(|&s| writer.write_sample(s))?;
        writer.finalize()?;
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    #[case::i8(
        vec![
            0xFF,
            0x80,
            0x00
        ],
        hound::WavSpec {
            channels: 1,
            sample_rate: 4000,
            bits_per_sample: 8,
            sample_format: hound::SampleFormat::Int,
        },
        &[i8::MAX, 0, i8::MIN]
    )]
    #[case::i16(
        vec![
            0xFF,
            0x80,
            0x00
        ],
        hound::WavSpec {
            channels: 1,
            sample_rate: 4000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        },
        &[i16::MAX, 0, i16::MIN]
    )]
    #[case::i24(
        vec![
            0xFF,
            0x80,
            0x00
        ],
        hound::WavSpec {
            channels: 1,
            sample_rate: 4000,
            bits_per_sample: 24,
            sample_format: hound::SampleFormat::Int,
        },
        &[8388607, 0, -8388608]
    )]
    #[case::i32(
        vec![
            0xFF,
            0x80,
            0x00
        ],
        hound::WavSpec {
            channels: 1,
            sample_rate: 4000,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Int,
        },
        &[i32::MAX, 0, i32::MIN]
    )]
    #[case::f32(
        vec![
            0xFF,
            0x80,
            0x00
        ],
        hound::WavSpec {
            channels: 1,
            sample_rate: 4000,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        },
        &[1., 0., -1.]
    )]
    fn test_wav(
        #[case] expect: Vec<u8>,
        #[case] spec: hound::WavSpec,
        #[case] data: &[impl hound::Sample + Copy],
    ) -> anyhow::Result<()> {
        let geometry = create_geometry(1);
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("tmp.wav");
        create_wav(&path, spec, data)?;
        let m = Wav::new(&path);
        assert_eq!(Ok(expect), m.calc(&geometry));

        Ok(())
    }

    #[test]
    fn test_wav_new_unsupported() -> anyhow::Result<()> {
        let geometry = create_geometry(1);
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("tmp.wav");
        create_wav(
            &path,
            hound::WavSpec {
                channels: 2,
                sample_rate: 4000,
                bits_per_sample: 32,
                sample_format: hound::SampleFormat::Int,
            },
            &[0, 0],
        )?;
        assert!(Wav::new(&path).calc(&geometry).is_err());
        Ok(())
    }

    #[test]
    fn test_wav_clone() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("tmp.wav");
        create_wav(
            &path,
            hound::WavSpec {
                channels: 1,
                sample_rate: 4000,
                bits_per_sample: 8,
                sample_format: hound::SampleFormat::Int,
            },
            &[i8::MAX, 0, i8::MIN],
        )?;
        let m = Wav::new(path);
        let m2 = m.clone();
        assert_eq!(m.sampling_config(), m2.sampling_config());
        Ok(())
    }
}
