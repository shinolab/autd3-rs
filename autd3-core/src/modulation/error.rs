use std::{convert::Infallible, time::Duration};

use derive_more::Display;
use derive_new::new;
use thiserror::Error;

use crate::defined::Freq;

#[derive(new, Error, Debug, Display, PartialEq, Clone)]
#[display("{}", msg)]
/// An error occurred during modulation calculation.
pub struct ModulationError {
    msg: String,
}

#[derive(Error, Debug, PartialEq, Copy, Clone)]
/// An error produced by the sampling configuration.
pub enum SamplingConfigError {
    /// Invalid sampling division.
    #[error("Sampling division must not be zero")]
    DivisionInvalid,
    /// Invalid sampling frequency.
    #[error("Sampling frequency ({0:?}) must divide the ultrasound frequency")]
    FreqInvalid(Freq<u32>),
    /// Invalid sampling frequency.
    #[error("Sampling frequency ({0:?}) must divide the ultrasound frequency")]
    FreqInvalidF(Freq<f32>),
    /// Invalid sampling period.
    #[error("Sampling period ({0:?}) must be a multiple of the ultrasound period")]
    PeriodInvalid(Duration),
    /// Sampling frequency is out of range.
    #[error("Sampling frequency ({0:?}) is out of range ([{1:?}, {2:?}])")]
    FreqOutOfRange(Freq<u32>, Freq<u32>, Freq<u32>),
    /// Sampling frequency is out of range.
    #[error("Sampling frequency ({0:?}) is out of range ([{1:?}, {2:?}])")]
    FreqOutOfRangeF(Freq<f32>, Freq<f32>, Freq<f32>),
    /// Sampling period is out of range.
    #[error("Sampling period ({0:?}) is out of range ([{1:?}, {2:?}])")]
    PeriodOutOfRange(Duration, Duration, Duration),
}

// GRCOV_EXCL_START
impl From<Infallible> for SamplingConfigError {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

impl From<Infallible> for ModulationError {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

impl From<SamplingConfigError> for ModulationError {
    fn from(e: SamplingConfigError) -> Self {
        Self::new(e.to_string())
    }
}
// GRCOV_EXCL_STOP
