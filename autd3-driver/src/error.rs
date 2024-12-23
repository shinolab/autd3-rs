use std::time::Duration;

use thiserror::Error;

use crate::{
    defined::{Freq, ULTRASOUND_FREQ, ULTRASOUND_PERIOD},
    firmware::cpu::GainSTMMode,
    firmware::fpga::*,
};

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
    #[error(
        "Silencer completion time ({0:?}) must be a multiple of {period:?}",
        period = ULTRASOUND_PERIOD
    )]
    InvalidSilencerCompletionTime(Duration),
    /// Silencer completion time is out of range.
    #[error(
        "Silencer completion time ({0:?}) is out of range ([{min:?}, {max:?}])",
        min = ULTRASOUND_PERIOD,
        max = ULTRASOUND_PERIOD * 256)]
    SilencerCompletionTimeOutOfRange(Duration),

    /// Unknown group key.
    #[error("Unknown group key({0})")]
    UnkownKey(String),
    /// Key is already used.
    #[error("Key({0}) is already used")]
    KeyIsAlreadyUsed(String),
    /// Unused group key.
    #[error("Unused group key({0})")]
    UnusedKey(String),

    /// Invalid sampling division.
    #[error("Sampling division ({0}) must not be zero")]
    SamplingDivisionInvalid(u16),
    /// Invalid sampling frequency.
    #[error("Sampling frequency ({0:?}) must divide {period:?}", period = ULTRASOUND_FREQ)]
    SamplingFreqInvalid(Freq<u32>),
    /// Invalid sampling frequency.
    #[error("Sampling frequency ({0:?}) must divide {period:?}", period = ULTRASOUND_FREQ)]
    SamplingFreqInvalidF(Freq<f32>),
    /// Invalid sampling period.
    #[error(
        "Sampling period ({0:?}) must be a multiple of {period:?}",
        period = ULTRASOUND_PERIOD
    )]
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

    /// Invalid STM period.
    #[error("STM sampling period ({1:?}/{0}) must be integer")]
    STMPeriodInvalid(usize, Duration),

    /// FociSTM buffer size is out of range.
    #[error(
        "FociSTM size ({0}) is out of range ([{min}, {max}])",
        min = STM_BUF_SIZE_MIN,
        max = FOCI_STM_BUF_SIZE_MAX
    )]
    FociSTMPointSizeOutOfRange(usize),
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
    ModulationError(String),
    /// Error in the gain.
    #[error("{0}")]
    GainError(String),
    /// Error in the Link.
    #[error("{0}")]
    LinkError(String),

    /// Link is closed.
    #[error("Link is closed")]
    LinkClosed,
    /// Failed to confirm the response from the device.
    #[error("Failed to confirm the response from the device")]
    ConfirmResponseFailed,
    /// Failed to send data.
    #[error("Failed to send data")]
    SendDataFailed,

    /// Invalid date time.
    #[error("The input data is invalid.")]
    InvalidDateTime,

    /// Error from Windows OS.
    #[cfg(target_os = "windows")]
    #[error("{0}")]
    WindowsError(#[from] windows::core::Error),

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
    /// Silencer cannot complete phase/intensity completion in the specified sampling period.
    #[error("Silencer cannot complete phase/intensity completion in the specified sampling period. Please lower the sampling frequency or make the completion time of Silencer longer than the sampling period.")]
    InvalidSilencerSettings,
}

impl AUTDDriverError {
    #[doc(hidden)]
    pub const fn firmware_err(ack: u8) -> Self {
        match ack {
            0x80 => AUTDDriverError::NotSupportedTag,
            0x81 => AUTDDriverError::InvalidMessageID,
            0x84 => AUTDDriverError::InvalidInfoType,
            0x85 => AUTDDriverError::InvalidGainSTMMode,
            0x88 => AUTDDriverError::InvalidSegmentTransition,
            0x8B => AUTDDriverError::MissTransitionTime,
            0x8E => AUTDDriverError::InvalidSilencerSettings,
            0x8F => AUTDDriverError::InvalidTransitionMode,
            _ => AUTDDriverError::UnknownFirmwareError(ack),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_unknown_firmware_err() {
        let err = AUTDDriverError::firmware_err(0xFF);
        assert!(err.source().is_none());
        assert_eq!(format!("{}", err), "Unknown firmware error: 255");
        assert_eq!(format!("{:?}", err), "UnknownFirmwareError(255)");
    }
}
