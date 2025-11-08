use autd3_core::link::Ack;

use crate::error::AUTDDriverError;

const NO_ERROR: u8 = 0x00;
const NOT_SUPPORTED_TAG: u8 = 0x01;
pub(crate) const INVALID_MESSAGE_ID: u8 = 0x02;
const INVALID_INFO_TYPE: u8 = 0x03;
const INVALID_GAIN_STM_MODE: u8 = 0x04;
const INVALID_SEGMENT_TRANSITION: u8 = 0x05;
const MISS_TRANSITION_TIME: u8 = 0x06;
const INVALID_SILENCER_SETTINGS: u8 = 0x07;
const INVALID_TRANSITION_MODE: u8 = 0x08;

#[doc(hidden)]
pub const fn check_firmware_err(ack: Ack) -> Result<(), AUTDDriverError> {
    match ack.err() {
        NO_ERROR => Ok(()),
        NOT_SUPPORTED_TAG => Err(AUTDDriverError::NotSupportedTag),
        INVALID_MESSAGE_ID => Err(AUTDDriverError::InvalidMessageID),
        INVALID_INFO_TYPE => Err(AUTDDriverError::InvalidInfoType),
        INVALID_GAIN_STM_MODE => Err(AUTDDriverError::InvalidGainSTMMode),
        INVALID_SEGMENT_TRANSITION => Err(AUTDDriverError::InvalidSegmentTransition),
        MISS_TRANSITION_TIME => Err(AUTDDriverError::MissTransitionTime),
        INVALID_SILENCER_SETTINGS => Err(AUTDDriverError::InvalidSilencerSettings),
        INVALID_TRANSITION_MODE => Err(AUTDDriverError::InvalidTransitionMode),
        _ => Err(AUTDDriverError::UnknownFirmwareError(ack.err())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case(AUTDDriverError::NotSupportedTag, NOT_SUPPORTED_TAG)]
    #[case(AUTDDriverError::InvalidMessageID, INVALID_MESSAGE_ID)]
    #[case(AUTDDriverError::InvalidInfoType, INVALID_INFO_TYPE)]
    #[case(AUTDDriverError::InvalidGainSTMMode, INVALID_GAIN_STM_MODE)]
    #[case(AUTDDriverError::InvalidSegmentTransition, INVALID_SEGMENT_TRANSITION)]
    #[case(AUTDDriverError::MissTransitionTime, MISS_TRANSITION_TIME)]
    #[case(AUTDDriverError::InvalidSilencerSettings, INVALID_SILENCER_SETTINGS)]
    #[case(AUTDDriverError::InvalidTransitionMode, INVALID_TRANSITION_MODE)]
    #[case(AUTDDriverError::UnknownFirmwareError(0x0F), 0xFF)]
    fn check_firmware_err(#[case] expected: AUTDDriverError, #[case] err_code: u8) {
        let ack = Ack::new(0, err_code);
        let result = super::check_firmware_err(ack);
        assert_eq!(result.unwrap_err(), expected);
    }
}
