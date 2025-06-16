use std::{convert::Infallible, time::Duration};

use autd3_core::{
    common::{FOCI_STM_FOCI_NUM_MIN, MOD_BUF_SIZE_MIN, STM_BUF_SIZE_MIN},
    datagram::CombinedError,
    derive::FirmwareLimits,
    gain::GainError,
    link::LinkError,
    modulation::ModulationError,
    sampling_config::SamplingConfigError,
};
use thiserror::Error;

/// A interface for error handling in autd3-driver.
#[derive(Error, Debug, PartialEq, Clone)]
#[non_exhaustive]
pub enum AUTDDriverError {
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

    /// Modulation buffer size is out of range.
    #[error("Modulation buffer size ({0}) is out of range ([{min}, {max}])", min = MOD_BUF_SIZE_MIN, max = .1.mod_buf_size_max)]
    ModulationSizeOutOfRange(usize, FirmwareLimits),

    /// FociSTM buffer size is out of range.
    #[error("The number of total foci ({0}) is out of range ([{min}, {max}])", min = STM_BUF_SIZE_MIN, max = .1.foci_stm_buf_size_max)]
    FociSTMTotalSizeOutOfRange(usize, FirmwareLimits),
    /// Number of foci is out of range.
    #[error("Number of foci ({0}) is out of range ([{min}, {max}])", min = FOCI_STM_FOCI_NUM_MIN, max = .1.num_foci_max)]
    FociSTMNumFociOutOfRange(usize, FirmwareLimits),
    /// FociSTM point is out of range.
    #[error(
        "Point coordinate ({0}, {1}, {2}) is out of range ([{min_x}, {max_x}], [{min_y}, {max_y}], [{min_z}, {max_z}])",
        min_x = .3.foci_stm_lower_x(),
        max_x = .3.foci_stm_upper_x(),
        min_y = .3.foci_stm_lower_y(),
        max_y = .3.foci_stm_upper_y(),
        min_z = .3.foci_stm_lower_z(),
        max_z = .3.foci_stm_upper_z()
    )]
    FociSTMPointOutOfRange(f32, f32, f32, FirmwareLimits),
    /// GainSTM buffer size is out of range.
    #[error("GainSTM size ({0}) is out of range ([{min}, {max}])", min = STM_BUF_SIZE_MIN, max = .1.gain_stm_buf_size_max)]
    GainSTMSizeOutOfRange(usize, FirmwareLimits),

    /// GPIO output type is not supported.
    #[error("GPIO output type ({0}) is not supported")]
    UnsupportedGPIOOutputType(String),

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

    /// Firmware version mismatch.
    #[error("Firmware version mismatch")]
    FirmwareVersionMismatch,

    /// Unsupported operation.
    #[error("Unsupported operation")]
    UnsupportedOperation,
    /// Unsupported firmware.
    #[error("Unsupported firmware")]
    UnsupportedFirmware,

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
