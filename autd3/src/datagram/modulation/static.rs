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
            config: SamplingConfig::DISABLE,
            loop_behavior: LoopBehavior::infinite(),
        }
    }

    pub fn with_intensity(intensity: impl Into<u8>) -> Self {
        Self {
            intensity: intensity.into(),
            config: SamplingConfig::DISABLE,
            loop_behavior: LoopBehavior::infinite(),
        }
    }
}

impl Modulation for Static {
    fn calc(&self, _: &Geometry) -> ModulationCalcResult {
        let intensity = self.intensity;
        Ok(vec![intensity; 2])
    }

    #[tracing::instrument(level = "debug", skip(_geometry))]
    fn trace(&self, _geometry: &Geometry) {
        tracing::info!("{}", tynm::type_name::<Self>());
    }
}

impl Default for Static {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::create_geometry;

    use super::*;

    #[test]
    fn test_static_default() {
        let geometry = create_geometry(1);
        let m = Static::default();
        assert_eq!(u8::MAX, m.intensity());
        assert_eq!(SamplingConfig::DISABLE, m.sampling_config());
        assert_eq!(Ok(vec![u8::MAX, u8::MAX]), m.calc(&geometry));
    }

    #[test]
    fn test_static_with_intensity() {
        let geometry = create_geometry(1);
        let m = Static::with_intensity(0x1F);
        assert_eq!(0x1F, m.intensity());
        assert_eq!(SamplingConfig::DISABLE, m.sampling_config());
        assert_eq!(Ok(vec![0x1F, 0x1F]), m.calc(&geometry));
    }
}
