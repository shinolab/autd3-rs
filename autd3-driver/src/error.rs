use std::time::Duration;

use thiserror::Error;

use crate::{
    defined::{Freq, ULTRASOUND_FREQ, ULTRASOUND_PERIOD},
    firmware::{cpu::GainSTMMode, fpga::*},
};

#[derive(Error, Debug, PartialEq, Clone)]
#[non_exhaustive]
pub enum AUTDInternalError {
    #[error(
        "Modulation buffer size ({0}) is out of range ([{}, {}])",
        MOD_BUF_SIZE_MIN,
        MOD_BUF_SIZE_MAX
    )]
    ModulationSizeOutOfRange(usize),

    #[error(
        "Silencer completion time ({0:?}) must be a multiple of {:?}",
        ULTRASOUND_PERIOD
    )]
    InvalidSilencerCompletionTime(Duration),
    #[error(
        "Silencer completion time ({0:?}) is out of range ([{:?}, {:?}])",
        ULTRASOUND_PERIOD,
        ULTRASOUND_PERIOD * 256)]
    SilencerCompletionTimeOutOfRange(Duration),

    #[error("Unknown group key: {0}")]
    UnkownKey(String),

    #[error("Sampling division ({0}) must not be zero")]
    SamplingDivisionInvalid(u16),
    #[error("Sampling frequency ({0:?}) must divide {:?}", ULTRASOUND_FREQ)]
    SamplingFreqInvalid(Freq<u32>),
    #[error("Sampling frequency ({0:?}) must divide {:?}", ULTRASOUND_FREQ)]
    SamplingFreqInvalidF(Freq<f32>),
    #[error(
        "Sampling period ({0:?}) must be a multiple of {:?}",
        ULTRASOUND_PERIOD
    )]
    SamplingPeriodInvalid(Duration),
    #[error("Sampling frequency ({0:?}) is out of range ([{1:?}, {2:?}])")]
    SamplingFreqOutOfRange(Freq<u32>, Freq<u32>, Freq<u32>),
    #[error("Sampling frequency ({0:?}) is out of range ([{1:?}, {2:?}])")]
    SamplingFreqOutOfRangeF(Freq<f32>, Freq<f32>, Freq<f32>),
    #[error("Sampling period ({0:?}) is out of range ([{1:?}, {2:?}])")]
    SamplingPeriodOutOfRange(Duration, Duration, Duration),

    #[error("STM sampling period ({1:?}/{0}) must be integer")]
    STMPeriodInvalid(usize, Duration),

    #[error(
        "FociSTM size ({0}) is out of range ([{}, {}])",
        STM_BUF_SIZE_MIN,
        FOCI_STM_BUF_SIZE_MAX
    )]
    FociSTMPointSizeOutOfRange(usize),
    #[error(
        "Number of foci ({0}) is out of range ([{}, {}])",
        1,
        FOCI_STM_FOCI_NUM_MAX
    )]
    FociSTMNumFociOutOfRange(usize),
    #[error(
        "Point coordinate ({0}, {1}, {2}) is out of range ([{}, {}], [{}, {}], [{}, {}])",
        FOCI_STM_FIXED_NUM_UNIT * FOCI_STM_FIXED_NUM_LOWER_X as f32,
        FOCI_STM_FIXED_NUM_UNIT * FOCI_STM_FIXED_NUM_UPPER_X as f32,
        FOCI_STM_FIXED_NUM_UNIT * FOCI_STM_FIXED_NUM_LOWER_Y as f32,
        FOCI_STM_FIXED_NUM_UNIT * FOCI_STM_FIXED_NUM_UPPER_Y as f32,
        FOCI_STM_FIXED_NUM_UNIT * FOCI_STM_FIXED_NUM_LOWER_Z as f32,
        FOCI_STM_FIXED_NUM_UNIT * FOCI_STM_FIXED_NUM_UPPER_Z as f32,
    )]
    FociSTMPointOutOfRange(f32, f32, f32),
    #[error(
        "GainSTM size ({0}) is out of range ([{}, {}])",
        STM_BUF_SIZE_MIN,
        GAIN_STM_BUF_SIZE_MAX
    )]
    GainSTMSizeOutOfRange(usize),

    #[error("GainSTMMode ({0:?}) is not supported")]
    GainSTMModeNotSupported(GainSTMMode),

    #[error("{0}")]
    ModulationError(String),
    #[error("{0}")]
    GainError(String),
    #[error("{0}")]
    LinkError(String),

    #[error("{0}")]
    NotSupported(String),

    #[error("Link is closed")]
    LinkClosed,
    #[error("Failed to confirm the response from the device")]
    ConfirmResponseFailed,
    #[error("Failed to send data")]
    SendDataFailed,

    #[error("The input data is invalid.")]
    InvalidDateTime,

    #[cfg(target_os = "windows")]
    #[error("{0}")]
    WindowsError(#[from] windows::core::Error),

    #[error("Not supported tag")]
    NotSupportedTag,
    #[error("Invalid message ID")]
    InvalidMessageID,
    #[error("Invalid info type")]
    InvalidInfoType,
    #[error("Invalid GainSTM mode")]
    InvalidGainSTMMode,
    #[error("Unknown firmware error: {0}")]
    UnknownFirmwareError(u8),
    #[error("Invalid segment transition")]
    InvalidSegmentTransition,
    #[error("Invalid transition mode")]
    InvalidTransitionMode,
    #[error("Miss transition time")]
    MissTransitionTime,
    #[error("Silencer cannot complete phase/intensity completion in the specified sampling period. Please lower the sampling frequency or make the completion time of Silencer longer than the sampling period.")]
    InvalidSilencerSettings,
}

impl AUTDInternalError {
    pub const fn firmware_err(ack: u8) -> Self {
        match ack {
            0x80 => AUTDInternalError::NotSupportedTag,
            0x81 => AUTDInternalError::InvalidMessageID,
            0x84 => AUTDInternalError::InvalidInfoType,
            0x85 => AUTDInternalError::InvalidGainSTMMode,
            0x88 => AUTDInternalError::InvalidSegmentTransition,
            0x8B => AUTDInternalError::MissTransitionTime,
            0x8E => AUTDInternalError::InvalidSilencerSettings,
            0x8F => AUTDInternalError::InvalidTransitionMode,
            _ => AUTDInternalError::UnknownFirmwareError(ack),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_unknown_firmware_err() {
        let err = AUTDInternalError::firmware_err(0xFF);
        assert!(err.source().is_none());
        assert_eq!(format!("{}", err), "Unknown firmware error: 255");
        assert_eq!(format!("{:?}", err), "UnknownFirmwareError(255)");
    }
}
