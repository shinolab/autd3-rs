use std::sync::Arc;

use crate::derive::*;

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
    fn calc(&self) -> ModulationCalcResult {
        let src = self.m.calc()?;
        Ok(Arc::new(
            src.iter()
                .map(|v| ((*v as f32 / 255.).sqrt() * 255.).round() as u8)
                .collect(),
        ))
    }

    #[tracing::instrument(level = "debug", skip(self, geometry), fields(%self.config, %self.loop_behavior))]
    // GRCOV_EXCL_START
    fn trace(&self, geometry: &Geometry) {
        tracing::debug!("{}", tynm::type_name::<Self>());
        <M as Modulation>::trace(&self.m, geometry);
    }
    // GRCOV_EXCL_STOP
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::{super::tests::TestModulation, *};
    use crate::firmware::fpga::IntoSamplingConfigNearest;

    use crate::defined::kHz;

    #[rstest::rstest]
    #[test]
    #[case::freq_4k((4. * kHz).into_sampling_config_nearest())]
    #[case::freq_8k((8. * kHz).into_sampling_config_nearest())]
    #[cfg_attr(miri, ignore)]
    fn test_sampling_config(#[case] config: SamplingConfig) {
        assert_eq!(
            config,
            TestModulation {
                buf: Arc::new(vec![u8::MIN; 2]),
                config,
                loop_behavior: LoopBehavior::infinite(),
            }
            .with_radiation_pressure()
            .sampling_config()
        );
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test() -> anyhow::Result<()> {
        let mut rng = rand::thread_rng();

        let buf = vec![rng.gen(), rng.gen()];
        assert_eq!(
            buf.iter()
                .map(|&x| ((x as f32 / 255.).sqrt() * 255.).round() as u8)
                .collect::<Vec<_>>(),
            *TestModulation {
                buf: Arc::new(buf.clone()),
                config: SamplingConfig::FREQ_4K,
                loop_behavior: LoopBehavior::infinite(),
            }
            .with_radiation_pressure()
            .calc()?
        );

        Ok(())
    }
}
