use crate::{derive::AUTDInternalError, fpga::FPGAState};

const READS_FPGA_STATE_ENABLED_BIT: u8 = 7;
const READS_FPGA_STATE_ENABLED: u8 = 1 << READS_FPGA_STATE_ENABLED_BIT;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct RxMessage {
    pub data: u8,
    pub ack: u8,
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
    use std::mem::size_of;

    use super::*;

    #[test]
    fn rx_message() {
        assert_eq!(size_of::<RxMessage>(), 2);
    }

    #[test]
    fn rx_message_clone() {
        let msg = RxMessage {
            ack: 0x01,
            data: 0x02,
        };
        let msg2 = msg;

        assert_eq!(msg.ack, msg2.ack);
        assert_eq!(msg.data, msg2.data);
    }

    #[test]
    fn rx_message_copy() {
        let msg = RxMessage {
            ack: 0x01,
            data: 0x02,
        };
        let msg2 = msg;

        assert_eq!(msg.ack, msg2.ack);
        assert_eq!(msg.data, msg2.data);
    }
}
