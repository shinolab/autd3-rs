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
    fn calc(&self, geometry: &Geometry) -> ModulationCalcResult {
        let src = self.m.calc(geometry)?;
        Ok(src
            .into_iter()
            .map(|v| ((v as f32 / 255.).sqrt() * 255.).round() as u8)
            .collect())
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

    use crate::{defined::kHz, geometry::tests::create_geometry};

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
        let geometry = create_geometry(1, 249);

        let mut rng = rand::thread_rng();

        let buf = vec![rng.gen(), rng.gen()];
        assert_eq!(
            buf.iter()
                .map(|&x| ((x as f32 / 255.).sqrt() * 255.).round() as u8)
                .collect::<Vec<_>>(),
            TestModulation {
                buf: buf.clone(),
                config: SamplingConfig::Freq(4 * kHz),
                loop_behavior: LoopBehavior::infinite(),
            }
            .with_radiation_pressure()
            .calc(&geometry)?
        );

        Ok(())
    }
}
