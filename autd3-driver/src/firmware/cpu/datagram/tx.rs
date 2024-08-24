use crate::{ethercat::EC_OUTPUT_FRAME_SIZE, firmware::cpu::Header};

use derive_more::{Deref, DerefMut};

const PAYLOAD_SIZE: usize = EC_OUTPUT_FRAME_SIZE - std::mem::size_of::<Header>();
type Payload = [u8; PAYLOAD_SIZE];

#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TxMessage {
    pub header: Header,
    pub payload: Payload,
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
                    payload: [0; PAYLOAD_SIZE],
                };
                num_devices
            ],
        }
    }

    pub fn total_len(&self) -> usize {
        self.data.len() * std::mem::size_of::<TxMessage>()
    }
}
