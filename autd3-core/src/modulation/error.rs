use std::time::Duration;

use derive_more::Display;
use derive_new::new;
use thiserror::Error;

use crate::defined::Freq;

#[derive(new, Error, Debug, Display, PartialEq, Clone)]
#[display("{}", msg)]
pub struct ModulationError {
    msg: String,
}

#[derive(Error, Debug, PartialEq, Clone)]
pub enum SamplingConfigError {
    /// Invalid sampling division.
    #[error("Sampling division ({0}) must not be zero")]
    SamplingDivisionInvalid(u16),
    /// Invalid sampling frequency.
    #[error("Sampling frequency ({0:?}) must divide theultrasound frequency")]
    SamplingFreqInvalid(Freq<u32>),
    /// Invalid sampling frequency.
    #[error("Sampling frequency ({0:?}) must divide the ultrasound frequency")]
    SamplingFreqInvalidF(Freq<f32>),
    /// Invalid sampling period.
    #[error("Sampling period ({0:?}) must be a multiple of the ultrasound period")]
    SamplingPeriodInvalid(Duration),
    /// Sampling frequency is out of range.
    #[error("Sampling frequency ({0:?}) is out of range ([{1:?}, {2:?}])")]
    SamplingFreqOutOfRange(Freq<u32>, Freq<u32>, Freq<u32>),
    /// Sampling frequency is out of range.
    #[error("Sampling frequency ({0:?}) is out of range ([{1:?}, {2:?}])")]
    SamplingFreqOutOfRangeF(Freq<f32>, Freq<f32>, Freq<f32>),
    /// Sampling period is out of range.
    #[error("Sampling period ({0:?}) is out of range ([{1:?}, {2:?}])")]
    SamplingPeriodOutOfRange(Duration, Duration, Duration),
}
