use autd3_driver::{common::EmitIntensity, derive::*};

/// Modulation for modulating radiation pressure instead of amplitude
#[derive(Modulation)]
pub struct RadiationPressure<M: Modulation> {
    m: M,
    #[no_change]
    config: SamplingConfiguration,
}

pub trait IntoRadiationPressure<M: Modulation> {
    /// Apply modulation to radiation pressure instead of amplitude
    fn with_radiation_pressure(self) -> RadiationPressure<M>;
}

impl<M: Modulation> IntoRadiationPressure<M> for M {
    fn with_radiation_pressure(self) -> RadiationPressure<M> {
        RadiationPressure {
            config: self.sampling_config(),
            m: self,
        }
    }
}

impl<M: Modulation> Modulation for RadiationPressure<M> {
    fn calc(&self) -> Result<Vec<EmitIntensity>, AUTDInternalError> {
        Ok(self
            .m
            .calc()?
            .iter()
            .map(|&v| EmitIntensity::new(((v.value() as float / 255.).sqrt() * 255.).round() as u8))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use crate::modulation::Sine;

    use super::*;

    #[test]
    fn test_radiation_impl() {
        let m = Sine::new(100.);
        let m_transformed = m.with_radiation_pressure();

        let vec = m.calc().unwrap();
        let vec_transformed = m_transformed.calc().unwrap();

        vec.iter().zip(&vec_transformed).for_each(|(&x, &y)| {
            assert_eq!(
                y.value(),
                ((x.value() as float / 255.).sqrt() * 255.).round() as u8
            );
        });

        assert_eq!(m.sampling_config(), m_transformed.sampling_config());
    }
}
