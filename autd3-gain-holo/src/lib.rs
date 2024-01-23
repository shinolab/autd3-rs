#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

mod amp;
mod backend;
mod backend_nalgebra;
mod combinatorial;
mod constraint;
mod error;
mod helper;
mod linear_synthesis;
mod matrix;
mod nls;

#[cfg(feature = "bench-utilities")]
pub mod bench_utilities;
#[cfg(feature = "test-utilities")]
pub mod test_utilities;

pub use backend::*;
pub use backend_nalgebra::NalgebraBackend;
pub use combinatorial::*;
pub use constraint::*;
pub use error::HoloError;
pub use linear_synthesis::*;
pub use matrix::*;
pub use nls::*;

pub use amp::{dB, Amplitude, Pascal};
