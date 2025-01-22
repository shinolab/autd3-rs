use autd3_core::derive::*;

/// [`Modulation`] for appling modulation to the radiation pressure instead of the acoustic pressure.
#[derive(Modulation, Debug)]
pub struct RadiationPressure<M: Modulation> {
    m: M,
}

impl<M: Modulation> RadiationPressure<M> {
    fn new(m: M) -> Self {
        Self { m }
    }
}

/// Trait to convert [`Modulation`] to [`RadiationPressure`].
pub trait IntoRadiationPressure<M: Modulation> {
    /// Convert [`Modulation`] to [`RadiationPressure`]
    fn into_radiation_pressure(self) -> RadiationPressure<M>;
}

impl<M: Modulation> IntoRadiationPressure<M> for M {
    fn into_radiation_pressure(self) -> RadiationPressure<M> {
        RadiationPressure::new(self)
    }
}

impl<M: Modulation> Modulation for RadiationPressure<M> {
    fn calc(self) -> Result<Vec<u8>, ModulationError> {
        let src = self.m.calc()?;
        Ok(src
            .iter()
            .map(|v| ((*v as f32 / 255.).sqrt() * 255.).round() as u8)
            .collect())
    }

    fn sampling_config(&self) -> Result<SamplingConfig, ModulationError> {
        self.m.sampling_config()
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
    fn test_sampling_config(#[case] config: SamplingConfig) {
        assert_eq!(
            Ok(config),
            Custom {
                buffer: vec![u8::MIN; 2],
                sampling_config: config,
                option: Default::default(),
            }
            .into_radiation_pressure()
            .sampling_config()
        );
    }

    #[test]
    fn test() -> anyhow::Result<()> {
        let mut rng = rand::thread_rng();

        let buf = vec![rng.gen(), rng.gen()];
        assert_eq!(
            buf.iter()
                .map(|&x| ((x as f32 / 255.).sqrt() * 255.).round() as u8)
                .collect::<Vec<_>>(),
            *Custom {
                buffer: buf.clone(),
                sampling_config: 4. * kHz,
                option: Default::default(),
            }
            .into_radiation_pressure()
            .calc()?
        );

        Ok(())
    }
}
