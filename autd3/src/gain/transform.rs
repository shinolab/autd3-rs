/*
 * File: transform.rs
 * Project: modulation
 * Created Date: 15/06/2023
 * Author: Shun Suzuki
 * -----
 * Last Modified: 16/01/2024
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2023 Shun Suzuki. All rights reserved.
 *
 */

use std::collections::HashMap;

use autd3_driver::{
    derive::*,
    geometry::{Device, Geometry},
};

/// Gain to transform gain data
#[derive(Gain)]
pub struct Transform<G: Gain + 'static, F: Fn(&Device, &Transducer, &Drive) -> Drive + 'static> {
    gain: G,
    f: F,
}

pub trait IntoTransform<G: Gain> {
    /// transform gain data
    ///
    /// # Arguments
    ///
    /// * `f` - transform function. The first argument is the device, the second is transducer, and the third is the original drive data.
    ///
    fn with_transform<F: Fn(&Device, &Transducer, &Drive) -> Drive>(self, f: F) -> Transform<G, F>;
}

impl<G: Gain> IntoTransform<G> for G {
    fn with_transform<F: Fn(&Device, &Transducer, &Drive) -> Drive>(self, f: F) -> Transform<G, F> {
        Transform { gain: self, f }
    }
}

impl<G: Gain + 'static, F: Fn(&Device, &Transducer, &Drive) -> Drive + 'static> Gain
    for Transform<G, F>
{
    fn calc(
        &self,
        geometry: &Geometry,
        filter: GainFilter,
    ) -> Result<HashMap<usize, Vec<Drive>>, AUTDInternalError> {
        Ok(self
            .gain
            .calc(geometry, filter)?
            .iter()
            .map(|(&k, v)| {
                (
                    k,
                    v.iter()
                        .enumerate()
                        .map(|(i, d)| (self.f)(&geometry[k], &geometry[k][i], d))
                        .collect::<Vec<_>>(),
                )
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::super::Uniform;
    use super::*;
    use autd3_driver::{
        autd3_device::AUTD3,
        geometry::{IntoDevice, Vector3},
    };

    #[test]
    fn test_gain_transform() {
        let geometry: Geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);

        let gain = Uniform::new(0x01).with_transform(|_, _, d| Drive {
            phase: Phase::new(0x80),
            intensity: d.intensity + EmitIntensity::new(0x80),
        });

        gain.calc(&geometry, GainFilter::All)
            .unwrap()
            .iter()
            .for_each(|(_, drive)| {
                drive.iter().for_each(|d| {
                    assert_eq!(d.phase, Phase::new(0x80));
                    assert_eq!(d.intensity, EmitIntensity::new(0x81));
                })
            });
    }

    #[test]
    fn test_gain_transform_derive() {
        let geometry: Geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);
        let gain = Uniform::new(0x01).with_transform(|_, _, d| Drive {
            phase: Phase::new(0x80),
            intensity: d.intensity + EmitIntensity::new(0x80),
        });
        let _ = gain.calc(&geometry, GainFilter::All).unwrap();
        let _ = gain.operation();
    }
}
