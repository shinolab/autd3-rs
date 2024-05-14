use crate::derive::*;

/// Modulation to transform modulation data
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
    /// transform modulation data
    ///
    /// # Arguments
    ///
    /// * `f` - transform function. The first argument is index of the element, and the second argument is the value of the element of the original modulation data.
    fn with_transform<F: Fn(usize, u8) -> u8>(self, f: F) -> Transform<M, F>;
}

impl<M: Modulation, F: Fn(usize, u8) -> u8> Modulation for Transform<M, F> {
    fn calc(&self, geometry: &Geometry) -> Result<Vec<u8>, AUTDInternalError> {
        Ok(self
            .m
            .calc(geometry)?
            .into_iter()
            .enumerate()
            .map(|(i, x)| (self.f)(i, x))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use crate::{defined::kHz, defined::FREQ_40K, geometry::tests::create_geometry};

    use super::{super::tests::TestModulation, *};

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
            .with_transform(|_, x| x) // GRCOV_EXCL_LINE
            .sampling_config()
        );
    }

    #[test]
    fn test() {
        let geometry = create_geometry(1, 249, FREQ_40K);

        let mut rng = rand::thread_rng();

        let buf = vec![rng.gen(), rng.gen()];

        assert_eq!(
            Ok(buf.iter().map(|&x| x / 2).collect::<Vec<_>>()),
            TestModulation {
                buf: buf.clone(),
                config: SamplingConfig::Freq(4 * kHz),
                loop_behavior: LoopBehavior::infinite(),
            }
            .with_transform(|_, x| x / 2)
            .calc(&geometry)
        );
    }
}
