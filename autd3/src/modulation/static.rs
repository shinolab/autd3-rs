use autd3_driver::derive::*;

/// Without modulation
#[derive(Modulation, Clone, Debug, PartialEq, Builder)]
pub struct Static {
    #[get]
    intensity: EmitIntensity,
    #[no_change]
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
}

impl Static {
    /// constructor
    pub const fn new() -> Self {
        Self {
            intensity: EmitIntensity::MAX,
            config: SamplingConfig::DISABLE,
            loop_behavior: LoopBehavior::infinite(),
        }
    }

    /// set intensity
    ///
    /// # Arguments
    ///
    /// * `intensity` - intensity
    ///
    pub fn with_intensity(intensity: impl Into<EmitIntensity>) -> Self {
        Self {
            intensity: intensity.into(),
            config: SamplingConfig::DISABLE,
            loop_behavior: LoopBehavior::infinite(),
        }
    }
}

impl Modulation for Static {
    fn calc(&self, _: &Geometry) -> Result<Vec<EmitIntensity>, AUTDInternalError> {
        Ok(vec![self.intensity; 2])
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
        assert_eq!(EmitIntensity::MAX, m.intensity());
        assert_eq!(SamplingConfig::DISABLE, m.sampling_config());
        assert_eq!(
            Ok(vec![EmitIntensity::MAX, EmitIntensity::MAX]),
            m.calc(&geometry)
        );
    }

    #[test]
    fn test_static_with_intensity() {
        let geometry = create_geometry(1);
        let m = Static::with_intensity(0x1F);
        assert_eq!(0x1F, m.intensity().value());
        assert_eq!(SamplingConfig::DISABLE, m.sampling_config());
        assert_eq!(
            Ok(vec![EmitIntensity::new(0x1F), EmitIntensity::new(0x1F)]),
            m.calc(&geometry)
        );
    }
}
