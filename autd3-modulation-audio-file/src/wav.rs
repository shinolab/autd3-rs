use autd3_core::{defined::Hz, derive::*};
use autd3_derive::Modulation;
use hound::SampleFormat;

use std::{fmt::Debug, path::Path};

use crate::error::AudioFileError;

/// [`Modulation`] from Wav data.
#[derive(Modulation, Debug)]
pub struct Wav {
    spec: hound::WavSpec,
    buffer: Vec<u8>,
}

impl Wav {
    /// Create a new [`Wav`].
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, AudioFileError> {
        let path = path.as_ref().to_path_buf();
        let mut reader = hound::WavReader::open(&path)?;
        let spec = reader.spec();
        tracing::debug!("wav spec: {:?}", spec);
        if spec.channels != 1 {
            return Err(AudioFileError::Wav(hound::Error::Unsupported));
        }
        let buffer = match spec.sample_format {
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

        Ok(Self { spec, buffer })
    }
}

impl Modulation for Wav {
    fn calc(self) -> Result<Vec<u8>, ModulationError> {
        Ok(self.buffer)
    }

    fn sampling_config(&self) -> SamplingConfig {
        SamplingConfig::new(self.spec.sample_rate as f32 * Hz)
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
        let m = Wav::new(path)?;
        assert_eq!(spec.sample_rate, m.sampling_config().freq()?.hz() as u32);
        assert_eq!(Ok(expect), m.calc());

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
        assert!(Wav::new(path).is_err());
        Ok(())
    }
}
