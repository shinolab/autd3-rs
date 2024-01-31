use autd3_driver::{common::EmitIntensity, derive::*};

/// Without modulation
#[derive(Modulation, Clone, Copy, PartialEq, Debug)]
pub struct Static {
    intensity: EmitIntensity,
    #[no_change]
    config: SamplingConfiguration,
}

impl Static {
    /// constructor
    pub const fn new() -> Self {
        Self {
            intensity: EmitIntensity::MAX,
            config: SamplingConfiguration::DISABLE,
        }
    }

    /// set emission intensity
    ///
    /// # Arguments
    ///
    /// * `intensity` - [EmitIntensity]
    ///
    pub fn with_intensity(intensity: impl Into<EmitIntensity>) -> Self {
        Self {
            intensity: intensity.into(),
            config: SamplingConfiguration::DISABLE,
        }
    }

    pub fn intensity(&self) -> EmitIntensity {
        self.intensity
    }
}

impl Modulation for Static {
    fn calc(&self) -> Result<Vec<EmitIntensity>, AUTDInternalError> {
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
    use super::*;

    #[test]
    fn test_static_default() -> anyhow::Result<()> {
        let m = Static::default();
        assert_eq!(EmitIntensity::MAX, m.intensity());
        assert_eq!(SamplingConfiguration::DISABLE, m.sampling_config());
        assert_eq!(vec![EmitIntensity::MAX, EmitIntensity::MAX], m.calc()?);

        Ok(())
    }

    #[test]
    fn test_static_with_intensity() -> anyhow::Result<()> {
        let m = Static::with_intensity(0x1F);
        assert_eq!(EmitIntensity::new(0x1F), m.intensity());
        assert_eq!(SamplingConfiguration::DISABLE, m.sampling_config());
        assert_eq!(
            vec![EmitIntensity::new(0x1F), EmitIntensity::new(0x1F)],
            m.calc()?
        );

        Ok(())
    }

    #[test]
    fn test_static_derive() {
        let m = Static::default();
        assert_eq!(m, m.clone());
    }
}
