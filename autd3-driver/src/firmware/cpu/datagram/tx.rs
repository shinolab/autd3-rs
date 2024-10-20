use crate::{ethercat::EC_OUTPUT_FRAME_SIZE, firmware::cpu::Header};

use derive_more::{Deref, DerefMut};
use zerocopy::IntoBytes;

const PAYLOAD_SIZE: usize = EC_OUTPUT_FRAME_SIZE - std::mem::size_of::<Header>();

#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq)]
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

#[derive(Clone, Deref, DerefMut, Debug, PartialEq, Eq)]
pub struct TxDatagram {
    #[deref]
    #[deref_mut]
    data: Vec<TxMessage>,
}

impl TxDatagram {
    pub fn new(num_devices: usize) -> Self {
        Self {
            data: vec![
                TxMessage {
                    header: Header {
                        msg_id: 0,
                        _pad: 0,
                        slot_2_offset: 0,
                    },
                    payload: [0u16; PAYLOAD_SIZE / size_of::<u16>()],
                };
                num_devices
            ],
        }
    }

    pub fn total_len(&self) -> usize {
        self.data.len() * std::mem::size_of::<TxMessage>()
    }
}
