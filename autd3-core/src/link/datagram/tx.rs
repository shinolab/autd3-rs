use zerocopy::{FromZeros, Immutable, IntoBytes};

use crate::ethercat::EC_OUTPUT_FRAME_SIZE;

use super::header::Header;

const PAYLOAD_SIZE: usize = EC_OUTPUT_FRAME_SIZE - core::mem::size_of::<Header>();

/// PDO output data representation
#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq, IntoBytes, Immutable, FromZeros)]
pub struct TxMessage {
    #[doc(hidden)]
    pub header: Header,
    payload: [u16; PAYLOAD_SIZE / size_of::<u16>()], // use u16 for alignment
}

impl TxMessage {
    #[doc(hidden)]
    #[must_use]
    pub fn payload(&self) -> &[u8] {
        self.payload.as_bytes()
    }

    #[doc(hidden)]
    #[must_use]
    pub fn payload_mut(&mut self) -> &mut [u8] {
        self.payload.as_mut_bytes()
    }
}
