use crate::{common::EmitIntensity, derive::*};

/// Modulation to transform modulation data
#[derive(Modulation)]
#[no_modulation_transform]
pub struct Transform<M: Modulation, F: Fn(usize, EmitIntensity) -> EmitIntensity> {
    m: M,
    #[no_change]
    config: SamplingConfiguration,
    f: F,
}

impl<M: Modulation, F: Fn(usize, EmitIntensity) -> EmitIntensity> Transform<M, F> {
    #[doc(hidden)]
    pub fn new(m: M, f: F) -> Self {
        Self {
            config: m.sampling_config(),
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
    use super::{super::tests::TestModulation, *};

    #[test]
    fn test_transform_impl() -> anyhow::Result<()> {
        let m = TestModulation {
            buf: vec![EmitIntensity::random(); 2],
            config: SamplingConfiguration::FREQ_4K_HZ,
        };
        let m_transformed = m.clone().with_transform(|_, x| x / 2);

        assert_eq!(
            m.calc()?.iter().map(|x| x / 2).collect::<Vec<_>>(),
            m_transformed.calc()?
        );
        assert_eq!(m.sampling_config(), m_transformed.sampling_config());

        Ok(())
    }
}
