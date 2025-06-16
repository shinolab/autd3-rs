use autd3_core::derive::*;

/// [`Modulation`] for applying modulation to the radiation pressure instead of the acoustic pressure.
#[derive(Modulation, Debug, Clone, Copy)]
pub struct RadiationPressure<M: Modulation> {
    /// The target [`Modulation`].
    pub target: M,
}

impl<M: Modulation> RadiationPressure<M> {
    /// Create a new [`RadiationPressure`].
    #[must_use]
    pub const fn new(target: M) -> Self {
        Self { target }
    }
}

impl<M: Modulation> Modulation for RadiationPressure<M> {
    fn calc(self, limits: &FirmwareLimits) -> Result<Vec<u8>, ModulationError> {
        let src = self.target.calc(limits)?;
        Ok(src
            .iter()
            .map(|v| ((*v as f32 / 255.).sqrt() * 255.).round() as u8)
            .collect())
    }

    fn sampling_config(&self) -> SamplingConfig {
        self.target.sampling_config()
    }
}

#[cfg(test)]
mod tests {
    use crate::modulation::Custom;
    use autd3_driver::common::kHz;
    use rand::Rng;

    use super::*;

    #[rstest::rstest]
    #[test]
    #[case::freq_4k(SamplingConfig::new(4. * kHz))]
    #[case::freq_8k(SamplingConfig::new(8. * kHz))]
    fn test_sampling_config(#[case] config: SamplingConfig) {
        assert_eq!(
            config,
            RadiationPressure::new(Custom {
                buffer: vec![u8::MIN; 2],
                sampling_config: config,
            })
            .sampling_config()
        );
    }

    #[test]
    fn test() -> anyhow::Result<()> {
        let mut rng = rand::rng();

        let buf = vec![rng.random(), rng.random()];
        assert_eq!(
            buf.iter()
                .map(|&x| ((x as f32 / 255.).sqrt() * 255.).round() as u8)
                .collect::<Vec<_>>(),
            *RadiationPressure {
                target: Custom {
                    buffer: buf.clone(),
                    sampling_config: 4. * kHz,
                }
            }
            .calc(&FirmwareLimits::unused())?
        );

        Ok(())
    }
}
