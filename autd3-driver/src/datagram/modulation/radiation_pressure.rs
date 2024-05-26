use crate::derive::*;

use super::ModCalcFn;

#[derive(Modulation)]
#[no_radiation_pressure]
#[no_modulation_transform]
pub struct RadiationPressure<M: Modulation> {
    m: M,
    #[no_change]
    config: SamplingConfig,
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
    fn with_radiation_pressure(self) -> RadiationPressure<M>;
}

impl<M: Modulation> Modulation for RadiationPressure<M> {
    fn calc<'a>(&'a self, geometry: &'a Geometry) -> Result<ModCalcFn<'a>, AUTDInternalError> {
        let src = self.m.calc(geometry)?;
        Ok(Box::new(move |dev| {
            Box::new(src(dev).map(|v| ((v as f64 / 255.).sqrt() * 255.).round() as u8))
        }))
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::{super::tests::TestModulation, *};

    use crate::{defined::kHz, defined::FREQ_40K, geometry::tests::create_geometry};

    #[rstest::rstest]
    #[test]
    #[case::freq_4k(SamplingConfig::Freq(4 * kHz))]
    #[case::disable(SamplingConfig::DISABLE)]
    fn test_sampling_config(#[case] config: SamplingConfig) {
        assert_eq!(
            config,
            TestModulation {
                buf: vec![u8::MIN; 2],
                config,
                loop_behavior: LoopBehavior::infinite(),
            }
            .with_radiation_pressure()
            .sampling_config()
        );
    }

    #[test]
    fn test() -> anyhow::Result<()> {
        let geometry = create_geometry(1, 249, FREQ_40K);

        let mut rng = rand::thread_rng();

        let buf = vec![rng.gen(), rng.gen()];
        geometry.devices().try_for_each(|dev| {
            assert_eq!(
                buf.iter()
                    .map(|&x| ((x as f64 / 255.).sqrt() * 255.).round() as u8)
                    .collect::<Vec<_>>(),
                TestModulation {
                    buf: buf.clone(),
                    config: SamplingConfig::Freq(4 * kHz),
                    loop_behavior: LoopBehavior::infinite(),
                }
                .with_radiation_pressure()
                .calc(&geometry)?(dev)
                .collect::<Vec<_>>()
            );
            Result::<(), AUTDInternalError>::Ok(())
        })?;

        Ok(())
    }
}
