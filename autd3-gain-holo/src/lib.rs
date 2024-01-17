/*
 * File: lib.rs
 * Project: src
 * Created Date: 28/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 17/01/2024
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Shun Suzuki. All rights reserved.
 *
 */

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
