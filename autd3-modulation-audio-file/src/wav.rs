use autd3_driver::{defined::Hz, derive::*};
use hound::SampleFormat;

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::error::AudioFileError;

#[derive(Modulation, Debug, Clone, PartialEq)]
pub struct Wav {
    path: PathBuf,
    #[no_change]
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
}

impl Wav {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, AudioFileError> {
        let path = path.as_ref().to_path_buf();
        let reader = hound::WavReader::open(&path)?;
        let spec = reader.spec();
        Ok(Self {
            path,
            config: (spec.sample_rate * Hz).try_into()?,
            loop_behavior: LoopBehavior::infinite(),
        })
    }

    fn read_buf(&self) -> Result<Vec<u8>, AudioFileError> {
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
                        .map(|i| (i - i8::MIN as i32) as _)
                        .collect(),
                    16 => raw_buffer
                        .iter()
                        .map(|i| ((i - i16::MIN as i32) as f32 / 257.).round() as _)
                        .collect(),
                    24 => raw_buffer
                        .iter()
                        .map(|i| ((i + 8388608i32) as f32 / 65793.).round() as _)
                        .collect(),
                    32 => raw_buffer
                        .iter()
                        .map(|&i| ((i as i64 - i32::MIN as i64) as f32 / 16843009.).round() as _)
                        .collect(),
                    _ => return Err(AudioFileError::Wav(hound::Error::Unsupported)), // GRCOV_EXCL_LINE
                }
            }
            SampleFormat::Float => {
                let raw_buffer = reader.samples::<f32>().collect::<Result<Vec<_>, _>>()?;
                match spec.bits_per_sample {
                    32 => raw_buffer
                        .iter()
                        .map(|&i| ((i + 1.0) / 2. * 255.).round() as _)
                        .collect(),
                    _ => return Err(AudioFileError::Wav(hound::Error::Unsupported)), // GRCOV_EXCL_LINE
                }
            }
        };

        Ok(buf)
    }
}

impl Modulation for Wav {
    #[allow(clippy::unnecessary_cast)]
    fn calc(&self) -> ModulationCalcResult {
        Ok(Arc::new(self.read_buf()?))
    }
}

#[cfg(test)]
mod tests {
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
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("tmp.wav");
        create_wav(&path, spec, data)?;
        let m = Wav::new(&path)?;
        assert_eq!(Ok(Arc::new(expect)), m.calc());

        Ok(())
    }

    #[test]
    fn test_wav_new_unsupported() -> anyhow::Result<()> {
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
        assert!(Wav::new(&path)?.calc().is_err());
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
        let m = Wav::new(path)?;
        let m2 = m.clone();
        assert_eq!(m.sampling_config(), m2.sampling_config());
        Ok(())
    }
}
