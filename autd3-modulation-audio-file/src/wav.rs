use autd3_driver::{common::EmitIntensity, derive::*};
use hound::SampleFormat;

use std::path::{Path, PathBuf};

use crate::error::AudioFileError;

/// Modulation constructed from wav file
///
/// The wav data is resampled to the sampling frequency of Modulation.
#[derive(Modulation, Clone, PartialEq, Debug)]
pub struct Wav {
    path: PathBuf,
    config: SamplingConfiguration,
    loop_behavior: LoopBehavior,
}

impl Wav {
    /// Constructor
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the wav file
    ///
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            config: SamplingConfiguration::FREQ_4K_HZ,
            loop_behavior: LoopBehavior::Infinite,
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
                        .map(|i| (i - std::i8::MIN as i32) as f32)
                        .collect(),
                    16 => raw_buffer
                        .iter()
                        .map(|i| (i - std::i16::MIN as i32) as f32 / 257.)
                        .collect(),
                    24 => raw_buffer
                        .iter()
                        .map(|i| (i + 8388608i32) as f32 / 65793.)
                        .collect(),
                    32 => raw_buffer
                        .iter()
                        .map(|&i| (i as i64 - std::i32::MIN as i64) as f32 / 16843009.)
                        .collect(),
                    _ => return Err(AudioFileError::Wav(hound::Error::Unsupported)),
                }
            }
            SampleFormat::Float => {
                let raw_buffer = reader.samples::<f32>().collect::<Result<Vec<_>, _>>()?;
                match spec.bits_per_sample {
                    32 => raw_buffer.iter().map(|&i| (i + 1.0) / 2. * 255.).collect(),
                    _ => return Err(AudioFileError::Wav(hound::Error::Unsupported)),
                }
            }
        };

        Ok((buf, spec.sample_rate))
    }
}

impl Modulation for Wav {
    #[allow(clippy::unnecessary_cast)]
    fn calc(&self) -> Result<Vec<EmitIntensity>, AUTDInternalError> {
        let (raw_buffer, sample_rate) = self.read_buf()?;
        Ok(wav_io::resample::linear(
            raw_buffer,
            1,
            sample_rate,
            self.sampling_config().frequency() as u32,
        )
        .iter()
        .map(|&d| EmitIntensity::new(d.round() as u8))
        .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_wav(
        path: impl AsRef<Path>,
        spec: hound::WavSpec,
        data: &[impl hound::Sample + Clone + Copy],
    ) -> anyhow::Result<()> {
        let mut writer = hound::WavWriter::create(path, spec)?;
        data.into_iter().try_for_each(|&s| writer.write_sample(s))?;
        writer.finalize()?;
        Ok(())
    }

    #[test]
    fn test_wav_new_i8() -> anyhow::Result<()> {
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
        assert_eq!(
            vec![
                EmitIntensity::new(0xFF),
                EmitIntensity::new(0x80),
                EmitIntensity::new(0x00)
            ],
            Wav::new(&path).calc()?
        );

        Ok(())
    }

    #[test]
    fn test_wav_new_i16() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("tmp.wav");
        create_wav(
            &path,
            hound::WavSpec {
                channels: 1,
                sample_rate: 4000,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            },
            &[i16::MAX, 0, i16::MIN],
        )?;
        assert_eq!(
            vec![
                EmitIntensity::new(0xFF),
                EmitIntensity::new(0x80),
                EmitIntensity::new(0x00)
            ],
            Wav::new(&path).calc()?
        );
        Ok(())
    }

    #[test]
    fn test_wav_new_i24() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("tmp.wav");
        create_wav(
            &path,
            hound::WavSpec {
                channels: 1,
                sample_rate: 4000,
                bits_per_sample: 24,
                sample_format: hound::SampleFormat::Int,
            },
            &[8388607, 0, -8388608],
        )?;
        assert_eq!(
            vec![
                EmitIntensity::new(0xFF),
                EmitIntensity::new(0x80),
                EmitIntensity::new(0x00)
            ],
            Wav::new(&path).calc()?
        );

        Ok(())
    }

    #[test]
    fn test_wav_new_i32() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("tmp.wav");
        create_wav(
            &path,
            hound::WavSpec {
                channels: 1,
                sample_rate: 4000,
                bits_per_sample: 32,
                sample_format: hound::SampleFormat::Int,
            },
            &[i32::MAX, 0, i32::MIN],
        )?;
        assert_eq!(
            vec![
                EmitIntensity::new(0xFF),
                EmitIntensity::new(0x80),
                EmitIntensity::new(0x00)
            ],
            Wav::new(&path).calc()?
        );

        Ok(())
    }

    #[test]
    fn test_wav_new_float() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("tmp.wav");
        create_wav(
            &path,
            hound::WavSpec {
                channels: 1,
                sample_rate: 4000,
                bits_per_sample: 32,
                sample_format: hound::SampleFormat::Float,
            },
            &[1., 0., -1.],
        )?;
        assert_eq!(
            vec![
                EmitIntensity::new(0xFF),
                EmitIntensity::new(0x80),
                EmitIntensity::new(0x00)
            ],
            Wav::new(&path).calc()?
        );

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
        assert!(Wav::new(&path).calc().is_err());
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
