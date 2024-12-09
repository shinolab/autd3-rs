use crate::{ethercat::EC_OUTPUT_FRAME_SIZE, firmware::cpu::Header};

use autd3_derive::Builder;
use derive_more::Display;
use zerocopy::{FromZeros, Immutable, IntoBytes};

const PAYLOAD_SIZE: usize = EC_OUTPUT_FRAME_SIZE - std::mem::size_of::<Header>();

#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq, IntoBytes, Immutable, FromZeros, Builder, Display)]
#[display("({:?}, TAG: {:#04X})", header, (payload[0] & 0xFF) as u8)]
pub struct TxMessage {
    #[get(ref, ref_mut)]
    header: Header,
    payload: [u16; PAYLOAD_SIZE / size_of::<u16>()], // use u16 for alignment
}

impl TxMessage {
    #[must_use]
    pub fn payload(&self) -> &[u8] {
        self.payload.as_bytes()
    }

    #[must_use]
    pub fn payload_mut(&mut self) -> &mut [u8] {
        self.payload.as_mut_bytes()
    }
}
