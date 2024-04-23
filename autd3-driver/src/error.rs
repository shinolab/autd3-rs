use thiserror::Error;

use crate::firmware::{fpga::*, operation::GainSTMMode};

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

    #[error("Sampling frequency division ({0}) is out of range ([{1}, {2}])")]
    SamplingFreqDivOutOfRange(u32, u32, u32),
    #[error("Sampling frequency ({0}) is out of range ([{1}, {2}])")]
    SamplingFreqOutOfRange(f64, f64, f64),
    #[error("Sampling period ({0} ns) is out of range ([{1}, {2}])")]
    SamplingPeriodOutOfRange(u128, u128, u128),

    #[error("STM frequency ({1} Hz, size={0}) is out of range ([{2}, {3}])")]
    STMFreqOutOfRange(usize, f64, f64, f64),
    #[error("STM period ({1} ns, size={0}) is out of range ([{2}, {3}])")]
    STMPeriodOutOfRange(usize, u128, usize, usize),

    #[error(
        "FocusSTM size ({0}) is out of range ([{}, {}])",
        STM_BUF_SIZE_MIN,
        FOCUS_STM_BUF_SIZE_MAX
    )]
    FocusSTMPointSizeOutOfRange(usize),
    #[error(
        "Point coordinate ({0}) is out of range ([{}, {}])",
        FOCUS_STM_FIXED_NUM_UNIT * FOCUS_STM_FIXED_NUM_LOWER as f64,
        FOCUS_STM_FIXED_NUM_UNIT * FOCUS_STM_FIXED_NUM_UPPER as f64,
    )]
    FocusSTMPointOutOfRange(f64),
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
    #[error("Frequency division is too small")]
    FrequencyDivisionTooSmall,
    #[error("Completion steps is too large")]
    CompletionStepsTooLarge,
    #[error("Invalid info type")]
    InvalidInfoType,
    #[error("Invalid GainSTM mode")]
    InvalidGainSTMMode,
    #[error("Unknown firmware error: {0}")]
    UnknownFirmwareError(u8),
    #[error("Invalid segment transition")]
    InvalidSegmentTransition,
    #[error("Invalid mode")]
    InvalidMode,
    #[error("Invalid pulse width encoder data size")]
    InvalidPulseWidthEncoderDataSize,
    #[error("Incomplete pulse width encoder table data")]
    IncompletePulseWidthEncoderData,
    #[error("Miss transition time")]
    MissTransitionTime,
}

impl AUTDInternalError {
    pub const fn firmware_err(ack: u8) -> Self {
        match ack {
            0x80 => AUTDInternalError::NotSupportedTag,
            0x81 => AUTDInternalError::InvalidMessageID,
            0x82 => AUTDInternalError::FrequencyDivisionTooSmall,
            0x83 => AUTDInternalError::CompletionStepsTooLarge,
            0x84 => AUTDInternalError::InvalidInfoType,
            0x85 => AUTDInternalError::InvalidGainSTMMode,
            0x87 => AUTDInternalError::InvalidMode,
            0x88 => AUTDInternalError::InvalidSegmentTransition,
            0x89 => AUTDInternalError::InvalidPulseWidthEncoderDataSize,
            0x8A => AUTDInternalError::IncompletePulseWidthEncoderData,
            0x8B => AUTDInternalError::MissTransitionTime,
            _ => AUTDInternalError::UnknownFirmwareError(ack),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_not_supported_tag() {
        let err = AUTDInternalError::firmware_err(0x80);
        assert!(err.source().is_none());
        assert_eq!(format!("{}", err), "Not supported tag");
        assert_eq!(format!("{:?}", err), "NotSupportedTag");
    }

    #[test]
    fn test_invalid_msg_id() {
        let err = AUTDInternalError::firmware_err(0x81);
        assert!(err.source().is_none());
        assert_eq!(format!("{}", err), "Invalid message ID");
        assert_eq!(format!("{:?}", err), "InvalidMessageID");
    }

    #[test]
    fn test_freq_div_too_small() {
        let err = AUTDInternalError::firmware_err(0x82);
        assert!(err.source().is_none());
        assert_eq!(format!("{}", err), "Frequency division is too small");
        assert_eq!(format!("{:?}", err), "FrequencyDivisionTooSmall");
    }

    #[test]
    fn test_completion_steps_too_large() {
        let err = AUTDInternalError::firmware_err(0x83);
        assert!(err.source().is_none());
        assert_eq!(format!("{}", err), "Completion steps is too large");
        assert_eq!(format!("{:?}", err), "CompletionStepsTooLarge");
    }
}
