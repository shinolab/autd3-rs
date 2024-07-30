use std::sync::Arc;

use crate::derive::*;

#[derive(Modulation)]
#[no_modulation_transform]
pub struct Transform<M: Modulation, F: Fn(usize, u8) -> u8> {
    m: M,
    #[no_change]
    config: SamplingConfig,
    f: F,
    loop_behavior: LoopBehavior,
}

impl<M: Modulation, F: Fn(usize, u8) -> u8> Transform<M, F> {
    #[doc(hidden)]
    pub fn new(m: M, f: F) -> Self {
        Self {
            config: m.sampling_config(),
            loop_behavior: m.loop_behavior(),
            m,
            f,
        }
    }
}

pub trait IntoTransform<M: Modulation> {
    fn with_transform<F: Fn(usize, u8) -> u8>(self, f: F) -> Transform<M, F>;
}

impl<M: Modulation, F: Fn(usize, u8) -> u8> Modulation for Transform<M, F> {
    fn calc(&self) -> ModulationCalcResult {
        let src = self.m.calc()?;
        Ok(Arc::new(
            src.iter()
                .enumerate()
                .map(|(i, x)| (self.f)(i, *x))
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

    use crate::defined::kHz;
    use crate::firmware::fpga::IntoSamplingConfigNearest;

    use super::{super::tests::TestModulation, *};

    #[rstest::rstest]
    #[test]
    #[case::freq_4k((4. * kHz).into_sampling_config_nearest())]
    #[case::freq_8k((8. * kHz).into_sampling_config_nearest())]
    #[cfg_attr(miri, ignore)]
    fn test_sampling_config(#[case] config: SamplingConfig) {
        use std::sync::Arc;

        assert_eq!(
            config,
            TestModulation {
                buf: Arc::new(vec![u8::MIN; 2]),
                config,
                loop_behavior: LoopBehavior::infinite(),
            }
            .with_transform(|_, x| x) // GRCOV_EXCL_LINE
            .sampling_config()
        );
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test() -> anyhow::Result<()> {
        let mut rng = rand::thread_rng();

        let buf = vec![rng.gen(), rng.gen()];
        assert_eq!(
            buf.iter().map(|&x| x / 2).collect::<Vec<_>>(),
            *TestModulation {
                buf: Arc::new(buf.clone()),
                config: SamplingConfig::FREQ_4K,
                loop_behavior: LoopBehavior::infinite(),
            }
            .with_transform(|_, x| x / 2)
            .calc()?
        );

        Ok(())
    }
}
