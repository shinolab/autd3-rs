use autd3_core::{common::Hz, derive::*};
use hound::SampleFormat;

use std::{fmt::Debug, path::Path};

use crate::error::AudioFileError;

/// [`Modulation`] from Wav data.
#[derive(Modulation, Debug, Clone)]
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

    /// Encode a [`Modulation`] into a mono 8-bit PCM WAV file.
    ///
    /// This writes the provided modulation's data into a wav file with:
    /// - `channels = 1`
    /// - `bits_per_sample = 8`
    /// - `sample_format = Int`
    /// - `sample_rate = sampling frequency of the modulation`
    ///
    /// The sample rate must be an integer number of hertz; otherwise this returns error.
    pub fn encode<P: AsRef<Path>, M: Modulation>(m: M, path: P) -> Result<(), AudioFileError> {
        let sample_rate = m.sampling_config().freq()?.hz();
        if !autd3_core::utils::float::is_integer(sample_rate as f64) {
            return Err(AudioFileError::Wav(hound::Error::Unsupported));
        }
        let sample_rate = sample_rate as u32;
        let buffer = m.calc(&FirmwareLimits {
            mod_buf_size_max: u32::MAX,
            ..FirmwareLimits::unused()
        })?;

        let spec = hound::WavSpec {
            channels: 1,
            sample_rate,
            bits_per_sample: 8,
            sample_format: SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(&path, spec)?;
        buffer
            .into_iter()
            .try_for_each(|b| writer.write_sample(b.wrapping_add(128) as i8))?;
        writer.finalize()?;
        Ok(())
    }
}

impl Modulation for Wav {
    fn calc(self, _: &FirmwareLimits) -> Result<Vec<u8>, ModulationError> {
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
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut writer = hound::WavWriter::create(path, spec)?;
        data.iter().try_for_each(|&s| writer.write_sample(s))?;
        writer.finalize()?;
        Ok(())
    }

    #[rstest::rstest]
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
    fn wav(
        #[case] expect: Vec<u8>,
        #[case] spec: hound::WavSpec,
        #[case] data: &[impl hound::Sample + Copy],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("tmp.wav");
        create_wav(&path, spec, data)?;
        let m = Wav::new(path)?;
        assert_eq!(spec.sample_rate, m.sampling_config().freq()?.hz() as u32);
        assert_eq!(Ok(expect), m.calc(&FirmwareLimits::unused()));

        Ok(())
    }

    #[test]
    fn wav_new_unsupported() -> Result<(), Box<dyn std::error::Error>> {
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

    #[test]
    fn encode_writes_expected_wav() -> Result<(), Box<dyn std::error::Error>> {
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

        let dir = tempfile::tempdir()?;
        let path = dir.path().join("enc.wav");
        let data = vec![0u8, 128u8, 255u8];
        let m = TestMod {
            data: data.clone(),
            rate: 4000.0,
        };
        Wav::encode(m, &path)?;

        let mut reader = hound::WavReader::open(&path)?;
        let spec = reader.spec();
        assert_eq!(spec.channels, 1);
        assert_eq!(spec.bits_per_sample, 8);
        assert_eq!(spec.sample_format, hound::SampleFormat::Int);
        assert_eq!(spec.sample_rate, 4000);

        let samples = reader.samples::<i8>().collect::<Result<Vec<_>, _>>()?;
        assert_eq!(samples, vec![-128, 0, 127]);

        let decoded = Wav::new(&path)?;
        assert_eq!(decoded.calc(&FirmwareLimits::unused())?, data);

        Ok(())
    }

    #[test]
    fn encode_rejects_non_integer_rate() -> Result<(), Box<dyn std::error::Error>> {
        struct TestMod;
        impl Modulation for TestMod {
            // GRCOV_EXCL_START
            fn calc(self, _: &FirmwareLimits) -> Result<Vec<u8>, ModulationError> {
                unreachable!()
            }
            // GRCOV_EXCL_STOP
            fn sampling_config(&self) -> SamplingConfig {
                SamplingConfig::new(std::num::NonZeroU16::new(3).unwrap())
            }
        }
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("enc_err.wav");
        let err = Wav::encode(TestMod, &path);
        match err {
            Err(AudioFileError::Wav(hound::Error::Unsupported)) => {}
            _ => panic!("unexpected error: {err:?}"), // GRCOV_EXCL_LINE
        }
        Ok(())
    }
}
