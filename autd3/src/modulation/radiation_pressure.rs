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
            .map(|&v| (((v.value() as float / 255.).sqrt() * 255.).round() as u8).into())
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use crate::modulation::Sine;

    use super::*;

    #[test]
    fn test_radiation_impl() -> anyhow::Result<()> {
        let m = Sine::new(100.);
        let m_transformed = m.with_radiation_pressure();

        assert_eq!(
            m.calc()?
                .iter()
                .map(|x| (((x.value() as float / 255.).sqrt() * 255.).round() as u8).into())
                .collect::<Vec<EmitIntensity>>(),
            m_transformed.calc()?
        );

        assert_eq!(m.sampling_config(), m_transformed.sampling_config());

        Ok(())
    }
}
