use crate::{derive::AUTDInternalError, firmware::fpga::FPGAState};

const READS_FPGA_STATE_ENABLED_BIT: u8 = 7;
const READS_FPGA_STATE_ENABLED: u8 = 1 << READS_FPGA_STATE_ENABLED_BIT;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct RxMessage {
    data: u8,
    ack: u8,
}

impl RxMessage {
    pub const fn new(ack: u8, data: u8) -> Self {
        Self { ack, data }
    }

    pub const fn ack(&self) -> u8 {
        self.ack
    }

    pub const fn data(&self) -> u8 {
        self.data
    }
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
    #[cfg_attr(miri, ignore)]
    fn test_message_size() {
        assert_eq!(2, size_of::<RxMessage>());
        assert_eq!(0, offset_of!(RxMessage, data));
        assert_eq!(1, offset_of!(RxMessage, ack));
    }
}
