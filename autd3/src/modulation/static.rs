/*
 * File: static.rs
 * Project: modulation
 * Created Date: 30/04/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 30/12/2023
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022-2023 Shun Suzuki. All rights reserved.
 *
 */

use autd3_derive::Modulation;

use autd3_driver::{common::EmitIntensity, derive::prelude::*};

/// Without modulation
#[derive(Modulation, Clone, Copy)]
pub struct Static {
    intensity: EmitIntensity,
    #[no_change]
    config: SamplingConfiguration,
}

impl Static {
    /// constructor
    pub fn new() -> Self {
        Self::with_intensity(EmitIntensity::MAX)
    }

    /// set emission intensity
    ///
    /// # Arguments
    ///
    /// * `intensity` - normalized emission intensity of the ultrasound (from 0 to 1)
    ///
    pub fn with_intensity<A: Into<EmitIntensity>>(intensity: A) -> Self {
        Self {
            intensity: intensity.into(),
            config: SamplingConfiguration::from_frequency_division(0xFFFFFFFF).unwrap(),
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
    fn test_static_default() {
        let m = Static::default();
        assert_eq!(m.intensity, EmitIntensity::MAX);
        assert_eq!(
            m.calc().unwrap(),
            vec![EmitIntensity::MAX, EmitIntensity::MAX]
        );
    }

    #[test]
    fn test_static_new() {
        let m = Static::new();
        assert_eq!(m.intensity, EmitIntensity::MAX);
        assert_eq!(
            m.calc().unwrap(),
            vec![EmitIntensity::MAX, EmitIntensity::MAX]
        );
    }

    #[test]
    fn test_static_with_intensity() {
        let m = Static::with_intensity(0x1F);
        assert_eq!(m.intensity, EmitIntensity::new(0x1F));
        assert_eq!(
            m.calc().unwrap(),
            vec![EmitIntensity::new(0x1F), EmitIntensity::new(0x1F)]
        );
    }
}
