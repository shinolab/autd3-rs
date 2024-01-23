use autd3_driver::{common::EmitIntensity, derive::*};

/// Modulation to transform modulation data
#[derive(Modulation)]
pub struct Transform<M: Modulation, F: Fn(usize, EmitIntensity) -> EmitIntensity> {
    m: M,
    #[no_change]
    config: SamplingConfiguration,
    f: F,
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

impl<M: Modulation> IntoTransform<M> for M {
    fn with_transform<F: Fn(usize, EmitIntensity) -> EmitIntensity>(self, f: F) -> Transform<M, F> {
        Transform {
            config: self.sampling_config(),
            f,
            m: self,
        }
    }
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
    use crate::modulation::Sine;

    use super::*;

    #[test]
    fn test_transform_impl() {
        let m = Sine::new(100.);
        let m_transformed = m.with_transform(|_, x| x / 2);

        let vec = m.calc().unwrap();
        let vec_transformed = m_transformed.calc().unwrap();

        vec.iter().zip(&vec_transformed).for_each(|(&x, &y)| {
            assert_eq!(y.value(), x.value() / 2);
        });

        assert_eq!(m.sampling_config(), m_transformed.sampling_config());
    }
}
