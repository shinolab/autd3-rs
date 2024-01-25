use autd3_driver::{common::EmitIntensity, derive::*};
use hound::{SampleFormat, WavSpec};

use std::path::{Path, PathBuf};

use crate::error::AudioFileError;

/// Modulation constructed from wav file
///
/// The wav data is resampled to the sampling frequency of Modulation.
#[derive(Modulation, Clone)]
pub struct Wav {
    path: PathBuf,
    config: SamplingConfiguration,
}

impl Wav {
    /// Constructor
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the wav file
    ///
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, AudioFileError> {
        Ok(Self {
            path: path.as_ref().to_path_buf(),
            config: SamplingConfiguration::FREQ_4K_HZ,
        })
    }

    fn read_buf(&self) -> Result<(Vec<f32>, WavSpec), AudioFileError> {
        let mut reader = hound::WavReader::open(&self.path)?;
        let spec = reader.spec();
        let raw_buffer = reader.samples::<i32>().collect::<Result<Vec<_>, _>>()?;
        Ok((
            match (spec.sample_format, spec.bits_per_sample) {
                (SampleFormat::Int, 8) => raw_buffer
                    .iter()
                    .map(|i| (i - std::i8::MIN as i32) as f32)
                    .collect(),
                (SampleFormat::Int, 16) => raw_buffer
                    .iter()
                    .map(|i| (i - std::i16::MIN as i32) as f32 / 257.)
                    .collect(),
                (SampleFormat::Int, 24) => raw_buffer
                    .iter()
                    .map(|i| (i + 8388608i32) as f32 / 65793.)
                    .collect(),
                (SampleFormat::Int, 32) => raw_buffer
                    .iter()
                    .map(|&i| (i as i64 - std::i32::MIN as i64) as f32 / 16843009.)
                    .collect(),
                _ => return Err(AudioFileError::Wav(hound::Error::Unsupported)),
            },
            spec,
        ))
    }
}

impl Modulation for Wav {
    #[allow(clippy::unnecessary_cast)]
    fn calc(&self) -> Result<Vec<EmitIntensity>, AUTDInternalError> {
        let (
            raw_buffer,
            WavSpec {
                channels,
                sample_rate,
                ..
            },
        ) = self.read_buf()?;
        Ok(wav_io::resample::linear(
            raw_buffer,
            channels,
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

    fn create_wav<P: AsRef<Path>, S: hound::Sample + Clone + Copy>(
        path: P,
        spec: hound::WavSpec,
        data: &[S],
    ) {
        std::fs::create_dir_all(path.as_ref().parent().unwrap()).unwrap();
        if path.as_ref().exists() {
            std::fs::remove_file(path.as_ref()).unwrap();
        }
        let mut writer = hound::WavWriter::create(path, spec).unwrap();
        data.into_iter()
            .for_each(|&s| writer.write_sample(s).unwrap());
        writer.finalize().unwrap();
    }

    #[test]
    fn test_wav_new_i8() {
        let home_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let path = Path::new(&home_dir).join("tmp").join("i8.wav");
        create_wav(
            &path,
            hound::WavSpec {
                channels: 1,
                sample_rate: 4000,
                bits_per_sample: 8,
                sample_format: hound::SampleFormat::Int,
            },
            &[i8::MAX, 0, i8::MIN],
        );
        let m = Wav::new(&path);
        assert!(m.is_ok());
        let m = m.unwrap();
        assert_eq!(
            m.calc().unwrap(),
            vec![
                EmitIntensity::new(0xFF),
                EmitIntensity::new(0x80),
                EmitIntensity::new(0x00)
            ]
        );
    }

    #[test]
    fn test_wav_new_i16() {
        let home_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let path = Path::new(&home_dir).join("tmp").join("i16.wav");
        create_wav(
            &path,
            hound::WavSpec {
                channels: 1,
                sample_rate: 4000,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            },
            &[i16::MAX, 0, i16::MIN],
        );
        let m = Wav::new(&path);
        assert!(m.is_ok());
        let m = m.unwrap();
        assert_eq!(
            m.calc().unwrap(),
            vec![
                EmitIntensity::new(0xFF),
                EmitIntensity::new(0x80),
                EmitIntensity::new(0x00)
            ]
        );
    }

    #[test]
    fn test_wav_new_i24() {
        let home_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let path = Path::new(&home_dir).join("tmp").join("i24.wav");
        create_wav(
            &path,
            hound::WavSpec {
                channels: 1,
                sample_rate: 4000,
                bits_per_sample: 24,
                sample_format: hound::SampleFormat::Int,
            },
            &[8388607, 0, -8388608],
        );
        let m = Wav::new(&path);
        assert!(m.is_ok());
        let m = m.unwrap();
        assert_eq!(
            m.calc().unwrap(),
            vec![
                EmitIntensity::new(0xFF),
                EmitIntensity::new(0x80),
                EmitIntensity::new(0x00)
            ]
        );
    }

    #[test]
    fn test_wav_new_i32() {
        let home_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let path = Path::new(&home_dir).join("tmp").join("i32.wav");
        create_wav(
            &path,
            hound::WavSpec {
                channels: 1,
                sample_rate: 4000,
                bits_per_sample: 32,
                sample_format: hound::SampleFormat::Int,
            },
            &[i32::MAX, 0, i32::MIN],
        );
        let m = Wav::new(&path);
        assert!(m.is_ok());
        let m = m.unwrap();
        assert_eq!(
            m.calc().unwrap(),
            vec![
                EmitIntensity::new(0xFF),
                EmitIntensity::new(0x80),
                EmitIntensity::new(0x00)
            ]
        );
    }

    #[test]
    fn test_wav_new_unsupported() {
        let home_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let path = Path::new(&home_dir).join("tmp").join("unsupported.wav");
        create_wav(
            &path,
            hound::WavSpec {
                channels: 1,
                sample_rate: 4000,
                bits_per_sample: 32,
                sample_format: hound::SampleFormat::Float,
            },
            &[0., 0., 0.],
        );
        let m = Wav::new(&path).unwrap();
        assert!(m.calc().is_err());
    }

    #[test]
    fn test_wav_clone() {
        let home_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let path = Path::new(&home_dir).join("tmp").join("clone.wav");
        create_wav(
            &path,
            hound::WavSpec {
                channels: 1,
                sample_rate: 4000,
                bits_per_sample: 8,
                sample_format: hound::SampleFormat::Int,
            },
            &[i8::MAX, 0, i8::MIN],
        );
        let m = Wav::new(path).unwrap();
        let m2 = m.clone();
        assert_eq!(m.sampling_config(), m2.sampling_config());
    }
}
