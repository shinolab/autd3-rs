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
            AUTDDriverError::InvalidSilencerCompletionTime(_)
            | AUTDDriverError::SilencerCompletionTimeOutOfRange(_)
            | AUTDDriverError::STMPeriodInvalid(_, _)
            | AUTDDriverError::ModulationSizeOutOfRange(_)
            | AUTDDriverError::FociSTMTotalSizeOutOfRange(_)
            | AUTDDriverError::FociSTMNumFociOutOfRange(_)
            | AUTDDriverError::FociSTMPointOutOfRange(_, _, _)
            | AUTDDriverError::GainSTMSizeOutOfRange(_)
            | AUTDDriverError::UnsupportedGPIOOutputType(_)
            | AUTDDriverError::UnknownKey(_)
            | AUTDDriverError::UnusedKey(_)
            | AUTDDriverError::ConfirmResponseFailed
            | AUTDDriverError::ReadFirmwareVersionFailed(_)
            | AUTDDriverError::InvalidDateTime
            | AUTDDriverError::FirmwareVersionMismatch
            | AUTDDriverError::UnsupportedOperation
            | AUTDDriverError::UnsupportedFirmware
            | AUTDDriverError::NotSupportedTag
            | AUTDDriverError::InvalidMessageID
            | AUTDDriverError::InvalidInfoType
            | AUTDDriverError::InvalidGainSTMMode
            | AUTDDriverError::UnknownFirmwareError(_)
            | AUTDDriverError::InvalidSegmentTransition
            | AUTDDriverError::InvalidTransitionMode
            | AUTDDriverError::MissTransitionTime
            | AUTDDriverError::InvalidSilencerSettings => None,
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

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::*;

    #[rstest::rstest]
    #[case(SamplingConfigError::FreqInvalid(1 * autd3_core::common::Hz), AUTDDriverError::SamplingConfig(SamplingConfigError::FreqInvalid(1 * autd3_core::common::Hz)))]
    #[case(
        PulseWidthError::PulseWidthOutOfRange(1, 1),
        AUTDDriverError::PulseWidth(PulseWidthError::PulseWidthOutOfRange(1, 1))
    )]
    #[case(
        ModulationError::new("test"),
        AUTDDriverError::Modulation(ModulationError::new("test"))
    )]
    #[case(GainError::new("test"), AUTDDriverError::Gain(GainError::new("test")))]
    #[case(LinkError::new("test"), AUTDDriverError::Link(LinkError::new("test")))]
    fn from<E>(#[case] source: E, #[case] expected: AUTDDriverError)
    where
        E: std::error::Error + Clone,
        AUTDDriverError: From<E>,
    {
        let err: AUTDDriverError = source.clone().into();
        assert_eq!(expected, err);
    }

    #[rstest::rstest]
    #[case(
        "Silencer completion time (1ms) must be a multiple of the ultrasound period",
        AUTDDriverError::InvalidSilencerCompletionTime(Duration::from_millis(1))
    )]
    #[case(
        "Silencer completion time (1ms) is out of range",
        AUTDDriverError::SilencerCompletionTimeOutOfRange(Duration::from_millis(1))
    )]
    #[case(
        "Sampling frequency (1 Hz) must divide the ultrasound frequency",
        AUTDDriverError::SamplingConfig(SamplingConfigError::FreqInvalid(1 * autd3_core::common::Hz))
    )]
    #[case(
        "STM sampling period (1ms/10) must be integer",
        AUTDDriverError::STMPeriodInvalid(10, Duration::from_millis(1))
    )]
    #[case(
        "Modulation buffer size (0) is out of range ([2, 65536])",
        AUTDDriverError::ModulationSizeOutOfRange(0)
    )]
    #[case(
        "The number of total foci (0) is out of range ([2, 65536])",
        AUTDDriverError::FociSTMTotalSizeOutOfRange(0)
    )]
    #[case(
        "Number of foci (0) is out of range ([1, 8])",
        AUTDDriverError::FociSTMNumFociOutOfRange(0)
    )]
    #[case(
        &format!("Point coordinate (1, 2, 3) is out of range ([{}, {}], [{}, {}], [{}, {}])", FOCI_STM_LOWER_X, FOCI_STM_UPPER_X, FOCI_STM_LOWER_Y, FOCI_STM_UPPER_Y, FOCI_STM_LOWER_Z, FOCI_STM_UPPER_Z),
        AUTDDriverError::FociSTMPointOutOfRange(1.0, 2.0, 3.0)
    )]
    #[case(
        "GainSTM size (0) is out of range ([2, 1024])",
        AUTDDriverError::GainSTMSizeOutOfRange(0)
    )]
    #[case(
        "GPIO output type (test) is not supported",
        AUTDDriverError::UnsupportedGPIOOutputType("test".to_string())
    )]
    #[case(
        "Pulse width (1) is out of range [0, 1)",
        AUTDDriverError::PulseWidth(PulseWidthError::PulseWidthOutOfRange(1, 1))
    )]
    #[case("test", AUTDDriverError::Modulation(ModulationError::new("test")))]
    #[case("test", AUTDDriverError::Gain(GainError::new("test")))]
    #[case("test", AUTDDriverError::Link(LinkError::new("test")))]
    #[case(
        "Unknown group key(test_key)",
        AUTDDriverError::UnknownKey("test_key".to_string())
    )]
    #[case(
        "Unused group key(test_key)",
        AUTDDriverError::UnusedKey("test_key".to_string())
    )]
    #[case(
        "Failed to confirm the response from the device",
        AUTDDriverError::ConfirmResponseFailed
    )]
    #[case(
        "Read firmware info failed: 0, 2",
        AUTDDriverError::ReadFirmwareVersionFailed(vec![false, true, false])
    )]
    #[case("The input data is invalid.", AUTDDriverError::InvalidDateTime)]
    #[case("Firmware version mismatch", AUTDDriverError::FirmwareVersionMismatch)]
    #[case("Unsupported operation", AUTDDriverError::UnsupportedOperation)]
    #[case("Unsupported firmware", AUTDDriverError::UnsupportedFirmware)]
    #[case("Not supported tag", AUTDDriverError::NotSupportedTag)]
    #[case("Invalid message ID", AUTDDriverError::InvalidMessageID)]
    #[case("Invalid info type", AUTDDriverError::InvalidInfoType)]
    #[case("Invalid GainSTM mode", AUTDDriverError::InvalidGainSTMMode)]
    #[case(
        "Unknown firmware error: 42",
        AUTDDriverError::UnknownFirmwareError(42)
    )]
    #[case(
        "Invalid segment transition",
        AUTDDriverError::InvalidSegmentTransition
    )]
    #[case("Invalid transition mode", AUTDDriverError::InvalidTransitionMode)]
    #[case("Miss transition time", AUTDDriverError::MissTransitionTime)]
    #[case(
        "Silencer cannot complete phase/intensity interpolation in the specified sampling period. Please lower the sampling frequency or make the completion time of Silencer longer than the sampling period of the AM/STM.",
        AUTDDriverError::InvalidSilencerSettings
    )]
    fn display(#[case] msg: &str, #[case] err: AUTDDriverError) {
        assert_eq!(msg, format!("{}", err))
    }

    #[rstest::rstest]
    #[case(
        false,
        AUTDDriverError::InvalidSilencerCompletionTime(Duration::from_millis(1))
    )]
    #[case(
        false,
        AUTDDriverError::SilencerCompletionTimeOutOfRange(Duration::from_millis(1))
    )]
    #[case(true, AUTDDriverError::SamplingConfig(SamplingConfigError::FreqInvalid(1 * autd3_core::common::Hz)))]
    #[case(false, AUTDDriverError::STMPeriodInvalid(10, Duration::from_millis(1)))]
    #[case(false, AUTDDriverError::ModulationSizeOutOfRange(0))]
    #[case(false, AUTDDriverError::FociSTMTotalSizeOutOfRange(0))]
    #[case(false, AUTDDriverError::FociSTMNumFociOutOfRange(0))]
    #[case(false, AUTDDriverError::FociSTMPointOutOfRange(1.0, 2.0, 3.0))]
    #[case(false, AUTDDriverError::GainSTMSizeOutOfRange(0))]
    #[case(false, AUTDDriverError::UnsupportedGPIOOutputType("test".to_string()))]
    #[case(
        true,
        AUTDDriverError::PulseWidth(PulseWidthError::PulseWidthOutOfRange(1, 1))
    )]
    #[case(true, AUTDDriverError::Modulation(ModulationError::new("test")))]
    #[case(true, AUTDDriverError::Gain(GainError::new("test")))]
    #[case(true, AUTDDriverError::Link(LinkError::new("test")))]
    #[case(false, AUTDDriverError::UnknownKey("test_key".to_string()))]
    #[case(false, AUTDDriverError::UnusedKey("test_key".to_string()))]
    #[case(false, AUTDDriverError::ConfirmResponseFailed)]
    #[case(false, AUTDDriverError::ReadFirmwareVersionFailed(vec![false, true, false]))]
    #[case(false, AUTDDriverError::InvalidDateTime)]
    #[case(false, AUTDDriverError::FirmwareVersionMismatch)]
    #[case(false, AUTDDriverError::UnsupportedOperation)]
    #[case(false, AUTDDriverError::UnsupportedFirmware)]
    #[case(false, AUTDDriverError::NotSupportedTag)]
    #[case(false, AUTDDriverError::InvalidMessageID)]
    #[case(false, AUTDDriverError::InvalidInfoType)]
    #[case(false, AUTDDriverError::InvalidGainSTMMode)]
    #[case(false, AUTDDriverError::UnknownFirmwareError(42))]
    #[case(false, AUTDDriverError::InvalidSegmentTransition)]
    #[case(false, AUTDDriverError::InvalidTransitionMode)]
    #[case(false, AUTDDriverError::MissTransitionTime)]
    #[case(false, AUTDDriverError::InvalidSilencerSettings)]
    fn source(#[case] has_source: bool, #[case] err: AUTDDriverError) {
        assert_eq!(has_source, err.source().is_some());
    }

    #[test]
    fn from_combined_error() {
        let mod_err = ModulationError::new("modulation error");
        let gain_err = GainError::new("gain error");
        let combined_mod: CombinedError<ModulationError, GainError> =
            CombinedError::E1(mod_err.clone());
        let combined_gain: CombinedError<ModulationError, GainError> =
            CombinedError::E2(gain_err.clone());

        assert_eq!(AUTDDriverError::Modulation(mod_err), combined_mod.into());
        assert_eq!(AUTDDriverError::Gain(gain_err), combined_gain.into());
    }
}
