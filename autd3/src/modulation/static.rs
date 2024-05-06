use autd3_driver::derive::*;

/// Without modulation
#[derive(Modulation, Clone, Debug, PartialEq, Builder)]
pub struct Static {
    #[get]
    intensity: u8,
    #[no_change]
    config: SamplingConfiguration,
    loop_behavior: LoopBehavior,
}

impl Static {
    /// constructor
    pub const fn new() -> Self {
        Self {
            intensity: u8::MAX,
            config: SamplingConfiguration::DISABLE,
            loop_behavior: LoopBehavior::infinite(),
        }
    }

    /// set emission intensity
    ///
    /// # Arguments
    ///
    /// * `intensity` - [u8]
    ///
    pub fn with_intensity(intensity: impl Into<u8>) -> Self {
        Self {
            intensity: intensity.into(),
            config: SamplingConfiguration::DISABLE,
            loop_behavior: LoopBehavior::infinite(),
        }
    }
}

impl Modulation for Static {
    fn calc(&self, geometry: &Geometry) -> Result<HashMap<usize, Vec<u8>>, AUTDInternalError> {
        Self::transform(geometry, |_| Ok(vec![self.intensity; 2]))
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
    fn test_static_default() -> anyhow::Result<()> {
        let geometry = create_geometry(1);
        let m = Static::default();
        assert_eq!(u8::MAX, m.intensity());
        assert_eq!(SamplingConfiguration::DISABLE, m.sampling_config());
        assert_eq!(vec![u8::MAX, u8::MAX], m.calc(&geometry)?[&0]);

        Ok(())
    }

    #[test]
    fn test_static_with_intensity() -> anyhow::Result<()> {
        let geometry = create_geometry(1);
        let m = Static::with_intensity(0x1F);
        assert_eq!(0x1F, m.intensity());
        assert_eq!(SamplingConfiguration::DISABLE, m.sampling_config());
        assert_eq!(vec![0x1F, 0x1F], m.calc(&geometry)?[&0]);

        Ok(())
    }

    #[test]
    fn test_static_derive() {
        let m = Static::default();
        assert_eq!(m, m.clone());
    }
}
