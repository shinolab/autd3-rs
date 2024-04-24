use crate::derive::*;

/// Modulation to transform modulation data
#[derive(Modulation)]
#[no_modulation_transform]
pub struct Transform<M: Modulation, F: Fn(usize, EmitIntensity) -> EmitIntensity> {
    m: M,
    #[no_change]
    config: SamplingConfiguration,
    f: F,
    loop_behavior: LoopBehavior,
}

impl<M: Modulation, F: Fn(usize, EmitIntensity) -> EmitIntensity> Transform<M, F> {
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
    ///
    /// # Example
    ///
    /// ```
    /// # use autd3::prelude::*;
    /// let m = Static::with_intensity(EmitIntensity::MAX);
    /// assert_eq!(m.calc(), Ok(vec![EmitIntensity::MAX, EmitIntensity::MAX]));
    /// let m = m.with_transform(|i, x| match i {
    ///     0 => x / 2,
    ///     _ => EmitIntensity::MIN,
    /// });
    /// assert_eq!(
    ///     m.calc(),
    ///     Ok(vec![EmitIntensity::MAX / 2, EmitIntensity::MIN])
    /// );
    /// ```
    fn with_transform<F: Fn(usize, EmitIntensity) -> EmitIntensity>(self, f: F) -> Transform<M, F>;
}

impl<M: Modulation, F: Fn(usize, EmitIntensity) -> EmitIntensity> Modulation for Transform<M, F> {
    fn calc(&self) -> Result<Vec<EmitIntensity>, AUTDInternalError> {
        Ok(self
            .m
            .calc()?
            .iter()
            .enumerate()
            .map(|(i, &x)| (self.f)(i, x))
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
    fn test_sampling_config(#[case] config: SamplingConfiguration) {
        assert_eq!(
            config,
            TestModulation {
                buf: vec![EmitIntensity::MIN; 2],
                config,
                loop_behavior: LoopBehavior::Infinite,
            }
            // GRCOV_EXCL_START
            .with_transform(|_, x| x)
            // GRCOV_EXCL_STOP
            .sampling_config()
        );
    }

    #[test]
    fn test() {
        let mut rng = rand::thread_rng();

        let buf = vec![rng.gen(), rng.gen()];

        assert_eq!(
            Ok(buf.iter().map(|&x| x / 2).collect::<Vec<_>>()),
            TestModulation {
                buf: buf.clone(),
                config: SamplingConfiguration::FREQ_4K_HZ,
                loop_behavior: LoopBehavior::Infinite,
            }
            .with_transform(|_, x| x / 2)
            .calc()
        );
    }
}
