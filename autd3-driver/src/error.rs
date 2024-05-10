use thiserror::Error;

use crate::{
    defined::Freq,
    firmware::{cpu::GainSTMMode, fpga::*},
};

#[derive(Error, Debug, PartialEq)]
pub enum AUTDInternalError {
    #[error(
        "Modulation buffer size ({0}) is out of range ([{}, {}])",
        MOD_BUF_SIZE_MIN,
        MOD_BUF_SIZE_MAX
    )]
    ModulationSizeOutOfRange(usize),

    #[error(
        "Silencer update rate ({0}) is out of range ([{}, {}])",
        SILENCER_VALUE_MIN,
        SILENCER_VALUE_MAX
    )]
    SilencerUpdateRateOutOfRange(u16),

    #[error(
        "Silencer completion steps ({0}) is out of range ([{}, {}])",
        SILENCER_VALUE_MIN,
        SILENCER_VALUE_MAX
    )]
    SilencerCompletionStepsOutOfRange(u16),

    #[error("Unknown group key: {0}")]
    UnkownKey(String),
    #[error("Unspecified group key: {0}")]
    UnspecifiedKey(String),

    #[error("Sampling frequency division ({0}) must be a multiple of 512")]
    SamplingFreqDivInvalid(u32),
    #[error("Sampling frequency division ({0}) is out of range ([{1}, {2}])")]
    SamplingFreqDivOutOfRange(u32, u32, u32),
    #[error("Sampling frequency ({0}) must divide {1}")]
    SamplingFreqInvalid(Freq<u32>, Freq<u32>),
    #[error("Sampling frequency ({0}) is out of range ([{1}, {2}])")]
    SamplingFreqOutOfRange(Freq<f64>, Freq<f64>, Freq<f64>),

    #[error("STM frequency ({1}, size={0}) must divide ultrasound frequency")]
    STMFreqInvalid(usize, Freq<f64>),

    #[error(
        "FocusSTM size ({0}) is out of range ([{}, {}])",
        STM_BUF_SIZE_MIN,
        FOCUS_STM_BUF_SIZE_MAX
    )]
    FocusSTMPointSizeOutOfRange(usize),
    #[error(
        "Point coordinate ({0}, {1}, {2}) is out of range ([{}, {}], [{}, {}], [{}, {}])",
        FOCUS_STM_FIXED_NUM_UNIT * FOCUS_STM_FIXED_NUM_LOWER_X as f64,
        FOCUS_STM_FIXED_NUM_UNIT * FOCUS_STM_FIXED_NUM_UPPER_X as f64,
        FOCUS_STM_FIXED_NUM_UNIT * FOCUS_STM_FIXED_NUM_LOWER_Y as f64,
        FOCUS_STM_FIXED_NUM_UNIT * FOCUS_STM_FIXED_NUM_UPPER_Y as f64,
        FOCUS_STM_FIXED_NUM_UNIT * FOCUS_STM_FIXED_NUM_LOWER_Z as f64,
        FOCUS_STM_FIXED_NUM_UNIT * FOCUS_STM_FIXED_NUM_UPPER_Z as f64,
    )]
    FocusSTMPointOutOfRange(f64, f64, f64),
    #[error(
        "GainSTM size ({0}) is out of range ([{}, {}])",
        STM_BUF_SIZE_MIN,
        GAIN_STM_BUF_SIZE_MAX
    )]
    GainSTMSizeOutOfRange(usize),

    #[error("GainSTMMode ({0:?}) is not supported")]
    GainSTMModeNotSupported(GainSTMMode),

    #[error("Invalid pulse width encoder table size ({0})")]
    InvalidPulseWidthEncoderTableSize(usize),
    #[error("Pulse width encoder table must be monotonically increasing and each data value must be 256 or less")]
    InvalidPulseWidthEncoderData,

    #[error("Frequency ({0}) can't be produced or is invalid for synchronizer")]
    InvalidFrequencyError(Freq<u32>),

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

    #[error("Failed to create timer")]
    TimerCreationFailed,
    #[error("Failed to delete timer")]
    TimerDeleteFailed,

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
    #[error("Invalid pulse width encoder data size")]
    InvalidPulseWidthEncoderDataSize,
    #[error("Incomplete pulse width encoder table data")]
    IncompletePulseWidthEncoderData,
    #[error("Incomplete DRP ROM data")]
    IncompleteDrpRomData,
    #[error("Miss transition time")]
    MissTransitionTime,
    #[error("Sampling frequency division is too small or silencer completion steps is too large")]
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
            0x89 => AUTDInternalError::InvalidPulseWidthEncoderDataSize,
            0x8A => AUTDInternalError::IncompletePulseWidthEncoderData,
            0x8B => AUTDInternalError::MissTransitionTime,
            0x8D => AUTDInternalError::IncompleteDrpRomData,
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
