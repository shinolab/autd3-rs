use std::sync::Arc;

use crate::derive::*;
use derive_more::Debug;

#[derive(Modulation, Debug)]
pub struct Transform<M: Modulation, F: Fn(usize, u8) -> u8> {
    m: M,
    #[debug(ignore)]
    f: F,
    #[no_change]
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
}

impl<M: Modulation, F: Fn(usize, u8) -> u8> Transform<M, F> {
    fn new(m: M, f: F) -> Self {
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

impl<M: Modulation> IntoTransform<M> for M {
    fn with_transform<F: Fn(usize, u8) -> u8>(self, f: F) -> Transform<M, F> {
        Transform::new(self, f)
    }
}

impl<M: Modulation, F: Fn(usize, u8) -> u8> Modulation for Transform<M, F> {
    fn calc(&self) -> Result<Arc<Vec<u8>>, AUTDInternalError> {
        let src = self.m.calc()?;
        Ok(Arc::new(
            src.iter()
                .enumerate()
                .map(|(i, x)| (self.f)(i, *x))
                .collect(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use crate::defined::kHz;

    use super::{super::tests::TestModulation, *};

    #[rstest::rstest]
    #[test]
    #[case::freq_4k(SamplingConfig::new_nearest(4. * kHz))]
    #[case::freq_8k(SamplingConfig::new_nearest(8. * kHz))]
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
