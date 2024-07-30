use std::num::NonZeroU16;

use autd3_driver::derive::*;

#[derive(Modulation, Clone, Debug, PartialEq, Builder)]
pub struct Static {
    #[get]
    intensity: u8,
    #[no_change]
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
}

impl Static {
    pub const fn new() -> Self {
        Self {
            intensity: u8::MAX,
            config: SamplingConfig::new(NonZeroU16::MAX),
            loop_behavior: LoopBehavior::infinite(),
        }
    }

    pub const fn with_intensity(intensity: u8) -> Self {
        Self {
            intensity,
            config: SamplingConfig::new(NonZeroU16::MAX),
            loop_behavior: LoopBehavior::infinite(),
        }
    }
}

impl Modulation for Static {
    fn calc(&self) -> ModulationCalcResult {
        let intensity = self.intensity;
        Ok(vec![intensity; 2])
    }

    #[tracing::instrument(level = "debug", skip(_geometry))]
    // GRCOV_EXCL_START
    fn trace(&self, _geometry: &Geometry) {
        tracing::debug!("{}", tynm::type_name::<Self>());
    }
    // GRCOV_EXCL_STOP
}

impl Default for Static {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_default() {
        let m = Static::default();
        assert_eq!(u8::MAX, m.intensity());
        assert_eq!(SamplingConfig::new(NonZeroU16::MAX), m.sampling_config());
        assert_eq!(Ok(vec![u8::MAX, u8::MAX]), m.calc());
    }

    #[test]
    fn test_static_with_intensity() {
        let m = Static::with_intensity(0x1F);
        assert_eq!(0x1F, m.intensity());
        assert_eq!(SamplingConfig::new(NonZeroU16::MAX), m.sampling_config());
        assert_eq!(Ok(vec![0x1F, 0x1F]), m.calc());
    }
}
