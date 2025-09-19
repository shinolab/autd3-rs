#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![warn(rustdoc::unescaped_backticks)]

//! This crate provides [`Gain`] that produces multiple focal points.
//!
//! [`Gain`]: autd3_core::gain::Gain

use nalgebra::{Dyn, U1, VecStorage};

#[allow(dead_code)]
pub(crate) type Complex = nalgebra::Complex<f32>;
#[allow(dead_code)]
pub(crate) type MatrixXc = nalgebra::Matrix<Complex, Dyn, Dyn, VecStorage<Complex, Dyn, Dyn>>;
#[allow(dead_code)]
pub(crate) type VectorXc = nalgebra::Matrix<Complex, Dyn, U1, VecStorage<Complex, Dyn, U1>>;

mod amp;
mod backend;
mod backend_nalgebra;
#[cfg(feature = "greedy")]
mod combinatorial;
mod constraint;
mod error;
#[cfg(any(feature = "naive", feature = "gs", feature = "gspat"))]
mod helper;
#[cfg(any(feature = "naive", feature = "gs", feature = "gspat"))]
mod linear_synthesis;

pub use backend::*;
pub use backend_nalgebra::NalgebraBackend;
#[cfg(feature = "greedy")]
pub use combinatorial::*;
pub use constraint::*;
pub use error::HoloError;
#[cfg(any(feature = "naive", feature = "gs", feature = "gspat"))]
pub use linear_synthesis::*;

pub use amp::{Amplitude, Pa, dB, kPa};
pub use autd3_core::acoustics::directivity::{Sphere, T4010A1};

#[cfg(test)]
pub(crate) mod tests {
    use autd3_core::geometry::{Geometry, Point3};
    use autd3_driver::autd3_device::AUTD3;

    pub fn create_geometry(row: usize, col: usize) -> Geometry {
        Geometry::new(
            (0..col)
                .flat_map(|i| {
                    (0..row).map(move |j| {
                        AUTD3 {
                            pos: Point3::new(i as f32 * 192., j as f32 * 151.4, 0.),
                            ..Default::default()
                        }
                        .into()
                    })
                })
                .collect(),
        )
    }
}
