use crate::{error::AUTDInternalError, firmware::fpga::FPGAState};
use autd3_derive::Builder;
use derive_more::Display;
use derive_new::new;
use zerocopy::{FromBytes, Immutable, IntoBytes};

const READS_FPGA_STATE_ENABLED_BIT: u8 = 7;
const READS_FPGA_STATE_ENABLED: u8 = 1 << READS_FPGA_STATE_ENABLED_BIT;

#[derive(
    Clone, Copy, PartialEq, Eq, Debug, new, Builder, IntoBytes, Immutable, FromBytes, Display,
)]
#[display("{:?}", self)]
#[repr(C)]
pub struct RxMessage {
    #[get]
    data: u8,
    #[get]
    ack: u8,
}

impl From<&RxMessage> for Option<FPGAState> {
    fn from(msg: &RxMessage) -> Self {
        if msg.data & READS_FPGA_STATE_ENABLED != 0 {
            Some(FPGAState { state: msg.data })
        } else {
            None
        }
    }
}

impl From<&RxMessage> for Result<(), AUTDInternalError> {
    fn from(msg: &RxMessage) -> Self {
        if msg.ack & 0x80 != 0 {
            return Err(AUTDInternalError::firmware_err(msg.ack));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::mem::offset_of;
    use std::mem::size_of;

    use super::*;

    #[test]
    fn test_message_size() {
        assert_eq!(2, size_of::<RxMessage>());
        assert_eq!(0, offset_of!(RxMessage, data));
        assert_eq!(1, offset_of!(RxMessage, ack));
    }
}
