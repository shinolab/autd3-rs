use crate::{ethercat::EC_OUTPUT_FRAME_SIZE, firmware::cpu::Header};

use zerocopy::{FromZeros, Immutable, IntoBytes};

const PAYLOAD_SIZE: usize = EC_OUTPUT_FRAME_SIZE - std::mem::size_of::<Header>();

#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq, IntoBytes, Immutable, FromZeros)]
pub struct TxMessage {
    header: Header,
    // use u16 for alignment
    payload: [u16; PAYLOAD_SIZE / size_of::<u16>()],
}

impl TxMessage {
    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn header_mut(&mut self) -> &mut Header {
        &mut self.header
    }

    pub fn payload(&self) -> &[u8] {
        self.payload.as_bytes()
    }

    pub fn payload_mut(&mut self) -> &mut [u8] {
        self.payload.as_mut_bytes()
    }
}
