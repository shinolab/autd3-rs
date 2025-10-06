use crate::{ethercat::EC_OUTPUT_FRAME_SIZE, link::MsgId};

use super::header::Header;

const PAYLOAD_SIZE: usize = EC_OUTPUT_FRAME_SIZE - core::mem::size_of::<Header>();

/// PDO output data representation
#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TxMessage {
    #[doc(hidden)]
    pub header: Header,
    payload: [u16; PAYLOAD_SIZE / size_of::<u16>()], // use u16 for alignment
}

impl TxMessage {
    #[doc(hidden)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            header: Header {
                msg_id: MsgId::new(0),
                __: 0x00,
                slot_2_offset: 0x00,
            },
            payload: [0; PAYLOAD_SIZE / size_of::<u16>()],
        }
    }

    #[doc(hidden)]
    #[must_use]
    pub fn payload(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.payload.as_ptr() as *const u8, PAYLOAD_SIZE) }
    }

    #[doc(hidden)]
    #[must_use]
    pub fn payload_mut(&mut self) -> &mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(self.payload.as_mut_ptr() as *mut u8, PAYLOAD_SIZE)
        }
    }
}
