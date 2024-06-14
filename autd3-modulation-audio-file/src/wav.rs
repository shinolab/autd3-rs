use autd3_driver::{defined::Hz, derive::*};
use hound::SampleFormat;

use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};

use crate::error::AudioFileError;

// TODO: Use `Cell` instead of `Mutex` for `config` field
// This is a breaking change because `Cell` is not `Sync` nor `RefUnwindSafe`
#[derive(Debug)]
pub struct Wav {
    path: PathBuf,
    config: Mutex<SamplingConfig>,
    loop_behavior: LoopBehavior,
}

impl Clone for Wav {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            config: Mutex::new(*self.config.lock().unwrap()),
            loop_behavior: self.loop_behavior,
        }
    }
}

impl PartialEq for Wav {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
            && *self.config.lock().unwrap() == *other.config.lock().unwrap()
            && self.loop_behavior == other.loop_behavior
    }
}

impl Wav {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            config: Mutex::new(SamplingConfig::DISABLE),
            loop_behavior: LoopBehavior::infinite(),
        }
    }

    fn read_buf(&self) -> Result<(Vec<u8>, u32), AudioFileError> {
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

        Ok((buf, spec.sample_rate))
    }

    // GRCOV_EXCL_START
    #[deprecated(note = "Do not change the sampling configuration", since = "25.3.0")]
    pub fn with_sampling_config(self, config: SamplingConfig) -> Self {
        Self {
            config: Mutex::new(config),
            ..self
        }
    }
    // GRCOV_EXCL_STOP

    pub fn with_loop_behavior(self, loop_behavior: LoopBehavior) -> Self {
        Self {
            loop_behavior,
            ..self
        }
    }
}

impl ModulationProperty for Wav {
    fn sampling_config(&self) -> SamplingConfig {
        *self.config.lock().unwrap()
    }

    fn loop_behavior(&self) -> LoopBehavior {
        self.loop_behavior
    }
}

impl Modulation for Wav {
    #[allow(clippy::unnecessary_cast)]
    fn calc(&self, _geometry: &Geometry) -> ModulationCalcResult {
        let (buf, sample_rate) = self.read_buf()?;
        *self.config.lock().unwrap() = SamplingConfig::Freq(sample_rate * Hz);
        Ok(buf)
    }

    #[tracing::instrument(level = "debug", skip(_geometry))]
    // GRCOV_EXCL_START
    fn trace(&self, _geometry: &Geometry) {
        tracing::info!("{}", tynm::type_name::<Self>());
    }
    // GRCOV_EXCL_STOP
}

impl DatagramST for Wav {
    type O1 = ModulationOp;
    type O2 = NullOp;
    type G = ModulationOperationGenerator;

    fn operation_generator_with_segment(
        self,
        geometry: &Geometry,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> Result<Self::G, AUTDInternalError> {
        Ok(Self::G {
            g: std::sync::Arc::new(self.calc(geometry)?),
            config: self.sampling_config(),
            rep: self.loop_behavior().rep(),
            segment,
            transition_mode,
        })
    }

    fn timeout(&self) -> Option<std::time::Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn parallel_threshold(&self) -> Option<usize> {
        Some(usize::MAX)
    }

    #[tracing::instrument(skip(self, geometry))]
    // GRCOV_EXCL_START
    fn trace(&self, geometry: &Geometry) {
        <Self as Modulation>::trace(self, geometry);
        if tracing::enabled!(tracing::Level::DEBUG) {
            if let Ok(buf) = <Self as Modulation>::calc(self, geometry) {
                if buf.is_empty() {
                    tracing::error!("Buffer is empty");
                    return;
                }
                if tracing::enabled!(tracing::Level::TRACE) {
                    buf.iter().enumerate().for_each(|(i, v)| {
                        tracing::debug!("Buf[{}]: {:#04X}", i, v);
                    });
                } else {
                    tracing::debug!("Buf[{}]: {:#04X}", 0, buf[0]);
                    if buf.len() > 2 {
                        tracing::debug!("︙");
                    }
                    if buf.len() > 1 {
                        tracing::debug!("Buf[{}]: {:#04X}", buf.len() - 1, buf.len() - 1);
                    }
                }
            } else {
                tracing::error!("Failed to calculate modulation");
            }
        }
    }
    // GRCOV_EXCL_STOP
}

impl IntoModulationTransform<Self> for Wav {
    fn with_transform<ModulationTransformF: Fn(usize, u8) -> u8>(
        self,
        f: ModulationTransformF,
    ) -> ModulationTransform<Self, ModulationTransformF> {
        ModulationTransform::new(self, f)
    }
}

impl IntoModulationCache<Self> for Wav {
    fn with_cache(self) -> ModulationCache<Self> {
        ModulationCache::new(self)
    }
}

impl IntoRadiationPressure<Self> for Wav {
    fn with_radiation_pressure(self) -> RadiationPressure<Self> {
        RadiationPressure::new(self)
    }
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
