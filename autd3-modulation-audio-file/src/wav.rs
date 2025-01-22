use autd3_core::{defined::Hz, derive::*, resampler::Resampler};
use autd3_derive::Modulation;
use hound::SampleFormat;

use std::{fmt::Debug, path::Path, rc::Rc};

use crate::error::AudioFileError;

/// The option of [`Wav`].
#[derive(Clone, Debug)]
pub struct WavOption {
    resampler: Option<(SamplingConfig, Rc<dyn Resampler>)>,
}

impl Default for WavOption {
    fn default() -> Self {
        Self { resampler: None }
    }
}

/// [`Modulation`] from Wav data.
#[derive(Modulation, Debug)]
pub struct Wav<'a> {
    /// The path to the Wav file.
    pub path: &'a Path,
    /// The option of [`Wav`].
    pub option: WavOption,
}

impl<'a> Wav<'a> {
    /// Resample the wav data to the target frequency.
    ///
    /// # Examples
    ///
    /// ```
    /// use autd3_core::{resampler::SincInterpolation, defined::kHz};
    /// use autd3_modulation_audio_file::Wav;
    ///
    /// let path = "path/to/file.csv";
    /// Wav {
    ///     path: std::path::Path::new(path),
    ///     option: Default::default(),
    /// }.with_resample(4 * kHz, SincInterpolation::default());
    /// ```
    pub fn with_resample<T>(
        self,
        target: T,
        resampler: impl Resampler + 'static,
    ) -> Result<Wav<'a>, T::Error>
    where
        T: TryInto<SamplingConfig>,
    {
        Ok(Wav {
            path: self.path,
            option: WavOption {
                resampler: Some((target.try_into()?, Rc::new(resampler))),
                ..self.option
            },
        })
    }
}

impl<'a> Wav<'a> {
    #[tracing::instrument]
    fn read_buf(&self) -> Result<(Vec<u8>, u32), AudioFileError> {
        let mut reader = hound::WavReader::open(&self.path)?;
        let spec = reader.spec();
        tracing::debug!("wav spec: {:?}", spec);
        if spec.channels != 1 {
            return Err(AudioFileError::Wav(hound::Error::Unsupported));
        }
        Ok((
            match spec.sample_format {
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
                            .map(|&i| {
                                ((i as i64 - i32::MIN as i64) as f32 / 16843009.).round() as _
                            })
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
            },
            spec.sample_rate,
        ))
    }
}

impl<'a> Modulation for Wav<'a> {
    fn calc(self) -> Result<Vec<u8>, ModulationError> {
        let (buffer, sample_rate) = self.read_buf()?;
        tracing::debug!("Read buffer: {:?}", buffer);
        Ok(if let Some((target, resampler)) = self.option.resampler {
            resampler.resample(&buffer, sample_rate as f32 * Hz, target)
        } else {
            buffer
        })
    }

    fn sampling_config(&self) -> Result<SamplingConfig, ModulationError> {
        if let Some((config, _)) = &self.option.resampler {
            Ok(*config)
        } else {
            let reader = hound::WavReader::open(&self.path).map_err(AudioFileError::from)?;
            let spec = reader.spec();
            Ok((spec.sample_rate * Hz).try_into()?)
        }
    }
}

#[cfg(test)]
mod tests {
    use autd3_core::{
        defined::{kHz, Freq},
        resampler::SincInterpolation,
    };

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
        let m = Wav {
            path: path.as_path(),
            option: WavOption::default(),
        };
        assert_eq!(Ok(expect), m.calc());

        Ok(())
    }

    #[rstest::rstest]
    #[case(vec![127, 217, 255, 217, 127, 37, 0, 37], vec![-1, 127, -1, -128], 2000, 4.0 * kHz, SincInterpolation::default())]
    #[case(vec![127, 255, 127, 0], vec![-1, 89, 127, 89, -1, -91, -128, -91], 8000, 4.0 * kHz, SincInterpolation::default())]
    #[test]
    fn new_with_resample(
        #[case] expected: Vec<u8>,
        #[case] buffer: Vec<i8>,
        #[case] sample_rate: u32,
        #[case] target: Freq<f32>,
        #[case] resampler: impl Resampler + 'static,
    ) -> anyhow::Result<()> {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tmp.csv");
        create_wav(
            &path,
            hound::WavSpec {
                channels: 1,
                sample_rate,
                bits_per_sample: 8,
                sample_format: hound::SampleFormat::Int,
            },
            &buffer,
        )?;

        let m = Wav {
            path: path.as_path(),
            option: WavOption::default(),
        }
        .with_resample(target, resampler)?;

        assert_eq!(expected, *m.calc()?);

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
        assert!(Wav {
            path: path.as_path(),
            option: WavOption::default(),
        }
        .calc()
        .is_err());
        Ok(())
    }
}
