use std::{convert::Infallible, time::Duration};

use autd3_core::{
    datagram::CombinedError,
    derive::SamplingConfigError,
    gain::GainError,
    link::{Ack, LinkError},
    modulation::ModulationError,
};
use thiserror::Error;

use crate::{firmware::cpu::GainSTMMode, firmware::fpga::*};

/// A interface for error handling in autd3-driver.
#[derive(Error, Debug, PartialEq, Clone)]
#[non_exhaustive]
pub enum AUTDDriverError {
    /// Modulation buffer size is out of range.
    #[error(
        "Modulation buffer size ({0}) is out of range ([{min}, {max}])",
        min = MOD_BUF_SIZE_MIN,
        max = MOD_BUF_SIZE_MAX
    )]
    ModulationSizeOutOfRange(usize),

    /// Invalid silencer completion time.
    #[error("Silencer completion time ({0:?}) must be a multiple of the ultrasound period")]
    InvalidSilencerCompletionTime(Duration),
    /// Silencer completion time is out of range.
    #[error("Silencer completion time ({0:?}) is out of range")]
    SilencerCompletionTimeOutOfRange(Duration),

    /// Sampling config error
    #[error("{0}")]
    SamplingConfig(#[from] SamplingConfigError),

    /// Invalid STM period.
    #[error("STM sampling period ({1:?}/{0}) must be integer")]
    STMPeriodInvalid(usize, Duration),

    /// FociSTM buffer size is out of range.
    #[error(
        "The number of total foci ({0}) is out of range ([{min}, {max}])",
        min = STM_BUF_SIZE_MIN,
        max = FOCI_STM_BUF_SIZE_MAX
    )]
    FociSTMTotalSizeOutOfRange(usize),
    /// Number of foci is out of range.
    #[error(
        "Number of foci ({0}) is out of range ([{min}, {max}])",
        min = 1,
        max = FOCI_STM_FOCI_NUM_MAX
    )]
    FociSTMNumFociOutOfRange(usize),
    /// FociSTM point is out of range.
    #[error(
        "Point coordinate ({0}, {1}, {2}) is out of range ([{x_min}, {x_max}], [{y_min}, {y_max}], [{z_min}, {z_max}])",
        x_min = FOCI_STM_FIXED_NUM_UNIT * FOCI_STM_FIXED_NUM_LOWER_X as f32,
        x_max = FOCI_STM_FIXED_NUM_UNIT * FOCI_STM_FIXED_NUM_UPPER_X as f32,
        y_min = FOCI_STM_FIXED_NUM_UNIT * FOCI_STM_FIXED_NUM_LOWER_Y as f32,
        y_max = FOCI_STM_FIXED_NUM_UNIT * FOCI_STM_FIXED_NUM_UPPER_Y as f32,
        z_min = FOCI_STM_FIXED_NUM_UNIT * FOCI_STM_FIXED_NUM_LOWER_Z as f32,
        z_max = FOCI_STM_FIXED_NUM_UNIT * FOCI_STM_FIXED_NUM_UPPER_Z as f32,
    )]
    FociSTMPointOutOfRange(f32, f32, f32),
    /// GainSTM buffer size is out of range.
    #[error(
        "GainSTM size ({0}) is out of range ([{min}, {max}])",
        min = STM_BUF_SIZE_MIN,
        max = GAIN_STM_BUF_SIZE_MAX
    )]
    GainSTMSizeOutOfRange(usize),
    /// GainSTM mode is not supported.
    #[error("GainSTMMode ({0:?}) is not supported")]
    GainSTMModeNotSupported(GainSTMMode),

    /// Error in the modulation.
    #[error("{0}")]
    Modulation(#[from] ModulationError),
    /// Error in the gain.
    #[error("{0}")]
    Gain(#[from] GainError),
    /// Error in the Link.
    #[error("{0}")]
    Link(#[from] LinkError),

    /// Unknown group key.
    #[error("Unknown group key({0})")]
    UnknownKey(String),
    /// Unused group key.
    #[error("Unused group key({0})")]
    UnusedKey(String),

    /// Failed to confirm the response from the device.
    #[error("Failed to confirm the response from the device")]
    ConfirmResponseFailed,

    /// Failed to read firmware version.
    #[error("Read firmware info failed: {}", .0.iter().enumerate().filter(|&(_, &b)| !b).map(|(i, _)| i.to_string()).collect::<Vec<_>>().join(", "))]
    ReadFirmwareVersionFailed(Vec<bool>),

    /// Invalid date time.
    #[error("The input data is invalid.")]
    InvalidDateTime,

    /// Not supported tag.
    ///
    /// Occurs when the software is not compatible with the firmware.
    #[error("Not supported tag")]
    NotSupportedTag,
    #[doc(hidden)]
    #[error("Invalid message ID")]
    InvalidMessageID,
    #[doc(hidden)]
    #[error("Invalid info type")]
    InvalidInfoType,
    #[doc(hidden)]
    #[error("Invalid GainSTM mode")]
    InvalidGainSTMMode,
    #[doc(hidden)]
    #[error("Unknown firmware error: {0}")]
    UnknownFirmwareError(u8),
    /// Invalid segment transition.
    #[error("Invalid segment transition")]
    InvalidSegmentTransition,
    /// Invalid segment transition mode.
    #[error("Invalid transition mode")]
    InvalidTransitionMode,
    /// Miss transition time.
    #[error("Miss transition time")]
    MissTransitionTime,
    /// Silencer cannot complete phase/intensity interpolation in the specified sampling period.
    #[error(
        "Silencer cannot complete phase/intensity interpolation in the specified sampling period. Please lower the sampling frequency or make the completion time of Silencer longer than the sampling period of the AM/STM."
    )]
    InvalidSilencerSettings,
}

#[doc(hidden)]
impl AUTDDriverError {
    pub const NO_ERROR: u8 = 0x00;
    pub const NOT_SUPPORTED_TAG: u8 = 0x01;
    pub const INVALID_MESSAGE_ID: u8 = 0x02;
    pub const INVALID_INFO_TYPE: u8 = 0x03;
    pub const INVALID_GAIN_STM_MODE: u8 = 0x04;
    pub const INVALID_SEGMENT_TRANSITION: u8 = 0x05;
    pub const MISS_TRANSITION_TIME: u8 = 0x06;
    pub const INVALID_SILENCER_SETTINGS: u8 = 0x07;
    pub const INVALID_TRANSITION_MODE: u8 = 0x08;

    pub const fn check_firmware_err(ack: Ack) -> Result<(), Self> {
        match ack.err() {
            Self::NO_ERROR => Ok(()),
            Self::NOT_SUPPORTED_TAG => Err(AUTDDriverError::NotSupportedTag),
            Self::INVALID_MESSAGE_ID => Err(AUTDDriverError::InvalidMessageID),
            Self::INVALID_INFO_TYPE => Err(AUTDDriverError::InvalidInfoType),
            Self::INVALID_GAIN_STM_MODE => Err(AUTDDriverError::InvalidGainSTMMode),
            Self::INVALID_SEGMENT_TRANSITION => Err(AUTDDriverError::InvalidSegmentTransition),
            Self::MISS_TRANSITION_TIME => Err(AUTDDriverError::MissTransitionTime),
            Self::INVALID_SILENCER_SETTINGS => Err(AUTDDriverError::InvalidSilencerSettings),
            Self::INVALID_TRANSITION_MODE => Err(AUTDDriverError::InvalidTransitionMode),
            _ => Err(AUTDDriverError::UnknownFirmwareError(ack.err())),
        }
    }
}

// GRCOV_EXCL_START
impl From<Infallible> for AUTDDriverError {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

impl<E1, E2> From<CombinedError<E1, E2>> for AUTDDriverError
where
    E1: std::error::Error,
    E2: std::error::Error,
    AUTDDriverError: From<E1> + From<E2>,
{
    fn from(err: CombinedError<E1, E2>) -> Self {
        match err {
            CombinedError::E1(e) => AUTDDriverError::from(e),
            CombinedError::E2(e) => AUTDDriverError::from(e),
        }
    }
}
// GRCOV_EXCL_STOP

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn unknown_firmware_err() {
        let err = AUTDDriverError::check_firmware_err(Ack::new().with_err(0x0F))
            .err()
            .unwrap();
        assert!(err.source().is_none());
        assert_eq!(format!("{}", err), "Unknown firmware error: 15");
        assert_eq!(format!("{:?}", err), "UnknownFirmwareError(15)");
    }
}
