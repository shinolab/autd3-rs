use crate::{ethercat::EC_OUTPUT_FRAME_SIZE, firmware::cpu::Header};

use autd3_derive::Builder;
use zerocopy::{FromZeros, Immutable, IntoBytes};

const PAYLOAD_SIZE: usize = EC_OUTPUT_FRAME_SIZE - std::mem::size_of::<Header>();

#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq, IntoBytes, Immutable, FromZeros, Builder)]
pub struct TxMessage {
    #[get(ref, ref_mut)]
    header: Header,
    #[as_bytes]
    payload: [u16; PAYLOAD_SIZE / size_of::<u16>()], // use u16 for alignment
}
