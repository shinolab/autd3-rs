use autd3_core::derive::*;
use autd3_derive::Modulation;
use derive_new::new;

/// [`Modulation`] for no modulation
#[derive(Modulation, Clone, Copy, Debug, PartialEq, new)]
pub struct Static {
    /// The intensity of the modulation. The default value is [`u8::MAX`].
    pub intensity: u8,
}

impl Modulation for Static {
    fn calc(self) -> Result<Vec<u8>, ModulationError> {
        let intensity = self.intensity;
        Ok(vec![intensity; 2])
    }

    fn sampling_config(&self) -> Result<SamplingConfig, ModulationError> {
        Ok(SamplingConfig::FREQ_MIN)
    }
}

impl Default for Static {
    fn default() -> Self {
        Self { intensity: u8::MAX }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_default() {
        let m = Static::default();
        assert_eq!(u8::MAX, m.intensity);
        assert_eq!(Ok(SamplingConfig::FREQ_MIN), m.sampling_config());
        assert_eq!(Ok(vec![u8::MAX, u8::MAX]), m.calc());
    }
}
