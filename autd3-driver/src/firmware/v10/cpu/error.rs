use autd3_core::link::Ack;

use crate::error::AUTDDriverError;

const NOT_SUPPORTED_TAG: u8 = 0x80;
const INVALID_MESSAGE_ID: u8 = 0x81;
const INVALID_INFO_TYPE: u8 = 0x84;
const INVALID_GAIN_STM_MODE: u8 = 0x85;
const INVALID_SEGMENT_TRANSITION: u8 = 0x88;
const MISS_TRANSITION_TIME: u8 = 0x8B;
const INVALID_SILENCER_SETTINGS: u8 = 0x8E;
const INVALID_TRANSITION_MODE: u8 = 0x8F;

#[doc(hidden)]
pub const fn check_firmware_err(ack: Ack) -> Result<(), AUTDDriverError> {
    let ack = ack.bits();
    if ack & 0x80 == 0x00 {
        return Ok(());
    }
    match ack {
        NOT_SUPPORTED_TAG => Err(AUTDDriverError::NotSupportedTag),
        INVALID_MESSAGE_ID => Err(AUTDDriverError::InvalidMessageID),
        INVALID_INFO_TYPE => Err(AUTDDriverError::InvalidInfoType),
        INVALID_GAIN_STM_MODE => Err(AUTDDriverError::InvalidGainSTMMode),
        INVALID_SEGMENT_TRANSITION => Err(AUTDDriverError::InvalidSegmentTransition),
        MISS_TRANSITION_TIME => Err(AUTDDriverError::MissTransitionTime),
        INVALID_SILENCER_SETTINGS => Err(AUTDDriverError::InvalidSilencerSettings),
        INVALID_TRANSITION_MODE => Err(AUTDDriverError::InvalidTransitionMode),
        _ => Err(AUTDDriverError::UnknownFirmwareError(ack)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn unknown_firmware_err() {
        let err = check_firmware_err(Ack::new(0x0F, 0x0F)).err().unwrap();
        assert!(err.source().is_none());
        assert_eq!(format!("{err}"), "Unknown firmware error: 255");
        assert_eq!(format!("{err:?}"), "UnknownFirmwareError(255)");
    }
}
