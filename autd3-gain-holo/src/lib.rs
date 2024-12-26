#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![warn(rustdoc::unescaped_backticks)]

//! This crate provides [`Gain`] that produces multiple focal points.
//!
//! [`Gain`]: autd3_driver::datagram::Gain

mod amp;
mod backend;
mod backend_nalgebra;
mod combinatorial;
mod constraint;
mod error;
mod helper;
mod linear_synthesis;
mod nls;

pub use backend::*;
pub use backend_nalgebra::NalgebraBackend;
pub use combinatorial::*;
pub use constraint::*;
pub use error::HoloError;
pub use linear_synthesis::*;
pub use nls::*;

pub use amp::{dB, kPa, Amplitude, Pa};
pub use autd3_driver::acoustics::directivity::{Sphere, T4010A1};
