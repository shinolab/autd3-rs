use std::{convert::Infallible, time::Duration};

use autd3_core::{
    datagram::CombinedError,
    firmware::{
        FOCI_STM_BUF_SIZE_MAX, FOCI_STM_FOCI_NUM_MAX, FOCI_STM_FOCI_NUM_MIN, FOCI_STM_LOWER_X,
        FOCI_STM_LOWER_Y, FOCI_STM_LOWER_Z, FOCI_STM_UPPER_X, FOCI_STM_UPPER_Y, FOCI_STM_UPPER_Z,
        GAIN_STM_BUF_SIZE_MAX, MOD_BUF_SIZE_MAX, MOD_BUF_SIZE_MIN, PulseWidthError,
        STM_BUF_SIZE_MIN, SamplingConfigError,
    },
    gain::GainError,
    link::LinkError,
    modulation::ModulationError,
};

/// A interface for error handling in autd3-driver.
#[derive(Debug, PartialEq, Clone)]
#[non_exhaustive]
pub enum AUTDDriverError {
    /// Invalid silencer completion time.
    InvalidSilencerCompletionTime(Duration),
    /// Silencer completion time is out of range.
    SilencerCompletionTimeOutOfRange(Duration),
    /// Sampling config error
    SamplingConfig(SamplingConfigError),

    /// Invalid STM period.
    STMPeriodInvalid(usize, Duration),

    /// Modulation buffer size is out of range.
    ModulationSizeOutOfRange(usize),

    /// FociSTM buffer size is out of range.
    FociSTMTotalSizeOutOfRange(usize),
    /// Number of foci is out of range.
    FociSTMNumFociOutOfRange(usize),
    /// FociSTM point is out of range.
    FociSTMPointOutOfRange(f32, f32, f32),
    /// GainSTM buffer size is out of range.
    GainSTMSizeOutOfRange(usize),

    /// GPIO output type is not supported.
    UnsupportedGPIOOutputType(String),

    /// Pulse width error.
    PulseWidth(PulseWidthError),

    /// Error in the modulation.
    Modulation(ModulationError),
    /// Error in the gain.
    Gain(GainError),
    /// Error in the Link.
    Link(LinkError),

    /// Unknown group key.
    UnknownKey(String),
    /// Unused group key.
    UnusedKey(String),

    /// Failed to confirm the response from the device.
    ConfirmResponseFailed,

    /// Failed to read firmware version.
    ReadFirmwareVersionFailed(Vec<bool>),

    /// Invalid date time.
    InvalidDateTime,

    /// Firmware version mismatch.
    FirmwareVersionMismatch,

    /// Unsupported operation.
    UnsupportedOperation,
    /// Unsupported firmware.
    UnsupportedFirmware,

    /// Not supported tag.
    ///
    /// Occurs when the software is not compatible with the firmware.
    NotSupportedTag,
    #[doc(hidden)]
    InvalidMessageID,
    #[doc(hidden)]
    InvalidInfoType,
    #[doc(hidden)]
    InvalidGainSTMMode,
    #[doc(hidden)]
    UnknownFirmwareError(u8),
    /// Invalid segment transition.
    InvalidSegmentTransition,
    /// Invalid transition mode.
    InvalidTransitionMode,
    /// Miss transition time.
    MissTransitionTime,
    /// Silencer cannot complete phase/intensity interpolation in the specified sampling period.
    InvalidSilencerSettings,
}

// GRCOV_EXCL_START
impl From<SamplingConfigError> for AUTDDriverError {
    fn from(e: SamplingConfigError) -> Self {
        AUTDDriverError::SamplingConfig(e)
    }
}

impl From<PulseWidthError> for AUTDDriverError {
    fn from(e: PulseWidthError) -> Self {
        AUTDDriverError::PulseWidth(e)
    }
}

impl From<ModulationError> for AUTDDriverError {
    fn from(e: ModulationError) -> Self {
        AUTDDriverError::Modulation(e)
    }
}

impl From<GainError> for AUTDDriverError {
    fn from(e: GainError) -> Self {
        AUTDDriverError::Gain(e)
    }
}

impl From<LinkError> for AUTDDriverError {
    fn from(e: LinkError) -> Self {
        AUTDDriverError::Link(e)
    }
}

impl std::fmt::Display for AUTDDriverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AUTDDriverError::InvalidSilencerCompletionTime(d) => write!(
                f,
                "Silencer completion time ({:?}) must be a multiple of the ultrasound period",
                d
            ),
            AUTDDriverError::SilencerCompletionTimeOutOfRange(d) => {
                write!(f, "Silencer completion time ({:?}) is out of range", d)
            }
            AUTDDriverError::SamplingConfig(e) => write!(f, "{}", e),
            AUTDDriverError::STMPeriodInvalid(size, period) => write!(
                f,
                "STM sampling period ({:?}/{}) must be integer",
                period, size
            ),
            AUTDDriverError::ModulationSizeOutOfRange(size) => write!(
                f,
                "Modulation buffer size ({}) is out of range ([{}, {}])",
                size, MOD_BUF_SIZE_MIN, MOD_BUF_SIZE_MAX
            ),
            AUTDDriverError::FociSTMTotalSizeOutOfRange(size) => write!(
                f,
                "The number of total foci ({}) is out of range ([{}, {}])",
                size, STM_BUF_SIZE_MIN, FOCI_STM_BUF_SIZE_MAX
            ),
            AUTDDriverError::FociSTMNumFociOutOfRange(size) => write!(
                f,
                "Number of foci ({}) is out of range ([{}, {}])",
                size, FOCI_STM_FOCI_NUM_MIN, FOCI_STM_FOCI_NUM_MAX
            ),
            AUTDDriverError::FociSTMPointOutOfRange(x, y, z) => write!(
                f,
                "Point coordinate ({}, {}, {}) is out of range ([{}, {}], [{}, {}], [{}, {}])",
                x,
                y,
                z,
                FOCI_STM_LOWER_X,
                FOCI_STM_UPPER_X,
                FOCI_STM_LOWER_Y,
                FOCI_STM_UPPER_Y,
                FOCI_STM_LOWER_Z,
                FOCI_STM_UPPER_Z,
            ),
            AUTDDriverError::GainSTMSizeOutOfRange(size) => write!(
                f,
                "GainSTM size ({}) is out of range ([{}, {}])",
                size, STM_BUF_SIZE_MIN, GAIN_STM_BUF_SIZE_MAX
            ),
            AUTDDriverError::UnsupportedGPIOOutputType(t) => {
                write!(f, "GPIO output type ({}) is not supported", t)
            }
            AUTDDriverError::PulseWidth(e) => write!(f, "{}", e),
            AUTDDriverError::Modulation(e) => write!(f, "{}", e),
            AUTDDriverError::Gain(e) => write!(f, "{}", e),
            AUTDDriverError::Link(e) => write!(f, "{}", e),
            AUTDDriverError::UnknownKey(key) => write!(f, "Unknown group key({})", key),
            AUTDDriverError::UnusedKey(key) => write!(f, "Unused group key({})", key),
            AUTDDriverError::ConfirmResponseFailed => {
                write!(f, "Failed to confirm the response from the device")
            }
            AUTDDriverError::ReadFirmwareVersionFailed(versions) => write!(
                f,
                "Read firmware info failed: {}",
                versions
                    .iter()
                    .enumerate()
                    .filter(|&(_, &b)| !b)
                    .map(|(i, _)| i.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            AUTDDriverError::InvalidDateTime => write!(f, "The input data is invalid."),
            AUTDDriverError::FirmwareVersionMismatch => write!(f, "Firmware version mismatch"),
            AUTDDriverError::UnsupportedOperation => write!(f, "Unsupported operation"),
            AUTDDriverError::UnsupportedFirmware => write!(f, "Unsupported firmware"),
            AUTDDriverError::NotSupportedTag => write!(f, "Not supported tag"),
            AUTDDriverError::InvalidMessageID => write!(f, "Invalid message ID"),
            AUTDDriverError::InvalidInfoType => write!(f, "Invalid info type"),
            AUTDDriverError::InvalidGainSTMMode => write!(f, "Invalid GainSTM mode"),
            AUTDDriverError::UnknownFirmwareError(e) => write!(f, "Unknown firmware error: {}", e),
            AUTDDriverError::InvalidSegmentTransition => write!(f, "Invalid segment transition"),
            AUTDDriverError::InvalidTransitionMode => write!(f, "Invalid transition mode"),
            AUTDDriverError::MissTransitionTime => write!(f, "Miss transition time"),
            AUTDDriverError::InvalidSilencerSettings => write!(
                f,
                "Silencer cannot complete phase/intensity interpolation in the specified sampling period. Please lower the sampling frequency or make the completion time of Silencer longer than the sampling period of the AM/STM."
            ),
        }
    }
}

impl std::error::Error for AUTDDriverError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AUTDDriverError::SamplingConfig(e) => Some(e),
            AUTDDriverError::PulseWidth(e) => Some(e),
            AUTDDriverError::Modulation(e) => Some(e),
            AUTDDriverError::Gain(e) => Some(e),
            AUTDDriverError::Link(e) => Some(e),
            _ => None,
        }
    }
}

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
