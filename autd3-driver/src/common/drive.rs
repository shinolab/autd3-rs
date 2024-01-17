/*
 * File: drive.rs
 * Project: common
 * Created Date: 14/10/2023
 * Author: Shun Suzuki
 * -----
 * Last Modified: 17/01/2024
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2023 Shun Suzuki. All rights reserved.
 *
 */

use super::{EmitIntensity, Phase};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Drive {
    /// Phase of ultrasound
    pub phase: Phase,
    /// emission intensity
    pub intensity: EmitIntensity,
}

impl Drive {
    pub const fn null() -> Self {
        Self {
            phase: Phase::new(0),
            intensity: EmitIntensity::MIN,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drive() {
        let d = Drive {
            phase: Phase::new(1),
            intensity: EmitIntensity::new(1),
        };

        let dc = Clone::clone(&d);
        assert_eq!(d.phase, dc.phase);
        assert_eq!(d.intensity, dc.intensity);

        assert_eq!(
            format!("{:?}", d),
            "Drive { phase: Phase { value: 1 }, intensity: EmitIntensity { value: 1 } }"
        );
    }
}
