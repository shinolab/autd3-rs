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
/// Primitive [`Gain`], [`Modulation`] and utilities for [`GainSTM`] and [`FociSTM`].
///
/// [`Gain`]: autd3_driver::datagram::Gain
/// [`Modulation`]: autd3_driver::datagram::Modulation
/// [`GainSTM`]: autd3_driver::datagram::GainSTM
/// [`FociSTM`]: autd3_driver::datagram::FociSTM
pub mod datagram;
/// Error module.
pub mod error;
/// Primitive [`Link`].
///
/// [`Link`]: autd3_driver::link::Link
pub mod link;
/// Prelude module.
pub mod prelude;

pub use autd3_driver as driver;
pub use autd3_driver::derive;
pub use datagram::gain;
pub use datagram::modulation;

pub use controller::Controller;

#[cfg(test)]
mod tests {
    use autd3_driver::{
        autd3_device::AUTD3,
        geometry::{Geometry, IntoDevice, Point3, Vector3},
    };

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

    pub fn random_vector3(
        range_x: std::ops::Range<f32>,
        range_y: std::ops::Range<f32>,
        range_z: std::ops::Range<f32>,
    ) -> Vector3 {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        Vector3::new(
            rng.gen_range(range_x),
            rng.gen_range(range_y),
            rng.gen_range(range_z),
        )
    }

    pub fn random_point3(
        range_x: std::ops::Range<f32>,
        range_y: std::ops::Range<f32>,
        range_z: std::ops::Range<f32>,
    ) -> Point3 {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        Point3::new(
            rng.gen_range(range_x),
            rng.gen_range(range_y),
            rng.gen_range(range_z),
        )
    }

    pub fn create_geometry(n: usize) -> Geometry {
        Geometry::new(
            (0..n)
                .map(|i| AUTD3::new(Point3::origin()).into_device(i as _))
                .collect(),
            4,
        )
    }
}
