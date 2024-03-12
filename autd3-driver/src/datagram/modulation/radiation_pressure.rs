use crate::derive::*;

/// Modulation for modulating radiation pressure instead of amplitude
#[derive(Modulation)]
#[no_radiation_pressure]
#[no_modulation_transform]
pub struct RadiationPressure<M: Modulation> {
    m: M,
    #[no_change]
    config: SamplingConfiguration,
    loop_behavior: LoopBehavior,
}

impl<M: Modulation> RadiationPressure<M> {
    #[doc(hidden)]
    pub fn new(m: M) -> Self {
        Self {
            config: m.sampling_config(),
            loop_behavior: m.loop_behavior(),
            m,
        }
    }
}

pub trait IntoRadiationPressure<M: Modulation> {
    /// Apply modulation to radiation pressure instead of amplitude
    fn with_radiation_pressure(self) -> RadiationPressure<M>;
}

impl<M: Modulation> Modulation for RadiationPressure<M> {
    fn calc(&self) -> Result<Vec<EmitIntensity>, AUTDInternalError> {
        Ok(self
            .m
            .calc()?
            .iter()
            .map(|v| (((v.value() as float / 255.).sqrt() * 255.).round() as u8).into())
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::{super::tests::TestModulation, *};

    #[rstest::rstest]
    #[test]
    #[case::freq_4k(SamplingConfiguration::FREQ_4K_HZ)]
    #[case::disable(SamplingConfiguration::DISABLE)]
    fn test_radiation_sampling_config(#[case] config: SamplingConfiguration) {
        assert_eq!(
            config,
            TestModulation {
                buf: vec![EmitIntensity::MIN; 2],
                config,
                loop_behavior: LoopBehavior::Infinite,
            }
            .with_radiation_pressure()
            .sampling_config()
        );
    }

    #[test]
    fn test_radiation() {
        let mut rng = rand::thread_rng();

        let buf = vec![rng.gen(), rng.gen()];
        assert_eq!(
            Ok(buf
                .iter()
                .map(
                    |x: &EmitIntensity| (((x.value() as float / 255.).sqrt() * 255.).round() as u8)
                        .into()
                )
                .collect::<Vec<EmitIntensity>>()),
            TestModulation {
                buf: buf.clone(),
                config: SamplingConfiguration::FREQ_4K_HZ,
                loop_behavior: LoopBehavior::Infinite,
            }
            .with_radiation_pressure()
            .calc()
        );
    }
}
