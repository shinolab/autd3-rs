use std::sync::Arc;

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
        Self::with_intensity(u8::MAX)
    }

    pub const fn with_intensity(intensity: u8) -> Self {
        Self {
            intensity,
            config: SamplingConfig::FREQ_MIN,
            loop_behavior: LoopBehavior::infinite(),
        }
    }
}

impl Modulation for Static {
    fn calc(&self) -> ModulationCalcResult {
        let intensity = self.intensity;
        Ok(Arc::new(vec![intensity; 2]))
    }
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
        assert_eq!(SamplingConfig::FREQ_MIN, m.sampling_config());
        assert_eq!(Ok(Arc::new(vec![u8::MAX, u8::MAX])), m.calc());
    }

    #[test]
    fn test_static_with_intensity() {
        let m = Static::with_intensity(0x1F);
        assert_eq!(0x1F, m.intensity());
        assert_eq!(SamplingConfig::FREQ_MIN, m.sampling_config());
        assert_eq!(Ok(Arc::new(vec![0x1F, 0x1F])), m.calc());
    }
}
