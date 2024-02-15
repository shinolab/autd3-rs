use crate::{common::EmitIntensity, derive::*};

/// Modulation for modulating radiation pressure instead of amplitude
#[derive(Modulation)]
#[no_radiation_pressure]
pub struct RadiationPressure<M: Modulation> {
    m: M,
    #[no_change]
    config: SamplingConfiguration,
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
    /// Apply modulation to radiation pressure instead of amplitude
    fn with_radiation_pressure(self) -> RadiationPressure<M>;
}

impl<M: Modulation> Modulation for RadiationPressure<M> {
    fn calc(&self) -> Result<Vec<EmitIntensity>, AUTDInternalError> {
        Ok(self
            .m
            .calc()?
            .iter()
            .map(|v| (((v.value() as float / 255.).sqrt() * 255.).round() as u8).into())
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::{super::tests::TestModulation, *};

    #[test]
    fn test_radiation_impl() -> anyhow::Result<()> {
        let m = TestModulation {
            buf: vec![EmitIntensity::random(); 2],
            config: SamplingConfiguration::FREQ_4K_HZ,
            loop_behavior: LoopBehavior::Infinite,
        };
        let m_transformed = m.clone().with_radiation_pressure();

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
