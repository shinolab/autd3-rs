use std::{convert::Infallible, time::Duration};

use thiserror::Error;

use crate::common::Freq;

#[derive(Error, Debug, PartialEq, Copy, Clone)]
/// An error produced by the sampling configuration.
pub enum SamplingConfigError {
    /// Invalid sampling divide.
    #[error("Sampling divide must not be zero")]
    DivideInvalid,
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
// GRCOV_EXCL_STOP
