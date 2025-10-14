#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![warn(rustdoc::unescaped_backticks)]

//! This crate provides [`Gain`] that produces multiple focal points.
//!
//! [`Gain`]: autd3_core::gain::Gain

#[cfg(feature = "use_nalgebra")]
#[cfg(any(feature = "naive", feature = "gs", feature = "gspat"))]
pub(crate) mod math {
    use autd3_core::geometry::Complex;
    use nalgebra::{Dyn, U1, VecStorage};

    pub(crate) type MatrixXc = nalgebra::Matrix<Complex, Dyn, Dyn, VecStorage<Complex, Dyn, Dyn>>;
    pub(crate) type VectorXc = nalgebra::Matrix<Complex, Dyn, U1, VecStorage<Complex, Dyn, U1>>;
    pub(crate) type RowVectorXc = nalgebra::Matrix<Complex, U1, Dyn, VecStorage<Complex, U1, Dyn>>;
}

#[cfg(not(feature = "use_nalgebra"))]
pub mod math;

#[cfg(any(feature = "naive", feature = "gs", feature = "gspat"))]
pub(crate) use math::*;

mod amp;
#[cfg(feature = "greedy")]
mod combinatorial;
mod constraint;
#[cfg(any(
    feature = "naive",
    feature = "gs",
    feature = "gspat",
    feature = "greedy"
))]
mod helper;
#[cfg(any(feature = "naive", feature = "gs", feature = "gspat"))]
mod linear_synthesis;

#[cfg(feature = "greedy")]
pub use combinatorial::*;
pub use constraint::*;
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
