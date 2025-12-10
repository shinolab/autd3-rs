#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![warn(rustdoc::unescaped_backticks)]

//! # AUTD3: Airborne Ultrasound Tactile Display 3
//!
//! Airborne Ultrasound Tactile Display (AUTD) is a midair haptic device that can remotely produce tactile sensation on a human skin surface without wearing devices.
//! Please see [our laboratory homepage](https://hapislab.org/en/airborne-ultrasound-tactile-display) for more details on AUTD.
//! This crate is a client library to drive AUTD version 3 devices. This cross-platform library supports Windows, macOS, and Linux (including Single Board Computer such as Raspberry Pi).

/// [`Controller`] module.
pub mod controller;
#[cfg_attr(docsrs, doc(cfg(feature = "gain")))]
#[cfg(feature = "gain")]
/// Primitive [`Gain`]s
///
/// [`Gain`]: autd3_core::gain::Gain
pub mod gain;
#[cfg_attr(docsrs, doc(cfg(feature = "link-nop")))]
/// Link modules.
pub mod link;
#[cfg_attr(docsrs, doc(cfg(feature = "modulation")))]
#[cfg(feature = "modulation")]
/// Primitive [`Modulation`]s
///
/// [`Modulation`]: autd3_core::modulation::Modulation
pub mod modulation;
/// Prelude module.
pub mod prelude;
#[cfg_attr(docsrs, doc(cfg(feature = "stm")))]
#[cfg(feature = "stm")]
/// Utilities for [`GainSTM`] and [`FociSTM`]
///
/// [`GainSTM`]: autd3_driver::datagram::GainSTM
/// [`FociSTM`]: autd3_driver::datagram::FociSTM
pub mod stm;

pub use autd3_core as core;
pub use autd3_driver as driver;
pub use controller::Controller;

#[cfg(test)]
mod tests {
    use autd3_core::{devices::AUTD3, geometry::UnitQuaternion};
    use autd3_driver::geometry::{Geometry, Point3, Vector3};

    #[macro_export]
    #[doc(hidden)]
    macro_rules! assert_near_vector3 {
        ($a:expr, $b:expr) => {
            let aa = $a;
            let bb = $b;
            let x = (aa.x - bb.x).abs() > 1e-3;
            let y = (aa.y - bb.y).abs() > 1e-3;
            let z = (aa.z - bb.z).abs() > 1e-3;
            if x || y || z {
                panic!(
                    "assertion failed: `(left ≈ right)`\n  left: `{:?}`,\n right: `{:?}`",
                    aa, bb
                );
            }
        };
    }

    #[must_use]
    pub fn random_vector3(
        range_x: std::ops::Range<f32>,
        range_y: std::ops::Range<f32>,
        range_z: std::ops::Range<f32>,
    ) -> Vector3 {
        use rand::Rng;
        let mut rng = rand::rng();
        Vector3::new(
            rng.random_range(range_x),
            rng.random_range(range_y),
            rng.random_range(range_z),
        )
    }

    #[must_use]
    pub fn random_point3(
        range_x: std::ops::Range<f32>,
        range_y: std::ops::Range<f32>,
        range_z: std::ops::Range<f32>,
    ) -> Point3 {
        use rand::Rng;
        let mut rng = rand::rng();
        Point3::new(
            rng.random_range(range_x),
            rng.random_range(range_y),
            rng.random_range(range_z),
        )
    }

    #[must_use]
    pub fn create_geometry(n: usize) -> Geometry {
        Geometry::new(
            (0..n)
                .map(|_| AUTD3::new(Point3::origin(), UnitQuaternion::identity()).into())
                .collect(),
        )
    }
}
