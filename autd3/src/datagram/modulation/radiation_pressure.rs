use std::sync::Arc;

use crate::derive::*;

#[derive(Modulation, Debug)]
pub struct RadiationPressure<M: Modulation> {
    m: M,
    #[no_change]
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
}

impl<M: Modulation> RadiationPressure<M> {
    fn new(m: M) -> Self {
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

impl<M: Modulation> IntoRadiationPressure<M> for M {
    fn with_radiation_pressure(self) -> RadiationPressure<M> {
        RadiationPressure::new(self)
    }
}

impl<M: Modulation> Modulation for RadiationPressure<M> {
    fn calc(&self) -> Result<Arc<Vec<u8>>, AUTDInternalError> {
        let src = self.m.calc()?;
        Ok(Arc::new(
            src.iter()
                .map(|v| ((*v as f32 / 255.).sqrt() * 255.).round() as u8)
                .collect(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::modulation::Custom;
    use autd3_driver::defined::kHz;
    use rand::Rng;

    use super::*;

    #[rstest::rstest]
    #[test]
    #[case::freq_4k(SamplingConfig::new_nearest(4. * kHz))]
    #[case::freq_8k(SamplingConfig::new_nearest(8. * kHz))]
    #[cfg_attr(miri, ignore)]
    fn test_sampling_config(#[case] config: SamplingConfig) {
        assert_eq!(
            config,
            Custom::new(&[u8::MIN; 2], config)
                .unwrap()
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
            *Custom::new(&buf, SamplingConfig::FREQ_4K)?
                .with_radiation_pressure()
                .calc()?
        );

        Ok(())
    }
}