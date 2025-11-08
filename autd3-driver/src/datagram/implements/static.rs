use std::num::NonZeroU16;

use autd3_core::derive::*;

/// [`Modulation`] for no modulation
#[derive(Modulation, Clone, Copy, Debug, PartialEq)]
pub struct Static {
    /// The intensity of the modulation. The default value is [`u8::MAX`].
    pub intensity: u8,
}

impl Static {
    /// Create a new [`Static`].
    #[must_use]
    pub const fn new(intensity: u8) -> Self {
        Self { intensity }
    }
}

impl Modulation for Static {
    fn calc(self) -> Result<Vec<u8>, ModulationError> {
        let intensity = self.intensity;
        Ok(vec![intensity; 2])
    }

    fn sampling_config(&self) -> SamplingConfig {
        SamplingConfig::Divide(NonZeroU16::MAX)
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
    fn default() {
        let m = Static::default();
        assert_eq!(u8::MAX, m.intensity);
        assert_eq!(SamplingConfig::Divide(NonZeroU16::MAX), m.sampling_config());
        assert_eq!(Ok(vec![u8::MAX, u8::MAX]), m.calc());
    }

    #[test]
    fn r#static() {
        let m = Static::new(u8::MIN);
        assert_eq!(u8::MIN, m.intensity);
        assert_eq!(SamplingConfig::Divide(NonZeroU16::MAX), m.sampling_config());
        assert_eq!(Ok(vec![u8::MIN, u8::MIN]), m.calc());
    }
}
