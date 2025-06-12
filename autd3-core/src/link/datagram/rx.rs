use derive_more::Display;
use getset::CopyGetters;
use zerocopy::{FromBytes, Immutable, IntoBytes};

/// Acknowledgement structure for received messages
#[bitfield_struct::bitfield(u8)]
#[derive(IntoBytes, Immutable, FromBytes, PartialEq, Eq, Display)]
pub struct Ack {
    #[bits(4)]
    pub msg_id: u8,
    #[bits(4)]
    pub err: u8,
}

/// PDO input data representation
#[derive(
    Clone, Copy, PartialEq, Eq, Debug, CopyGetters, IntoBytes, Immutable, FromBytes, Display,
)]
#[display("{:?}", self)]
#[repr(C)]
pub struct RxMessage {
    #[getset(get_copy = "pub")]
    /// Received data
    data: u8,
    #[getset(get_copy = "pub")]
    /// Acknowledgement
    ack: Ack,
}

impl RxMessage {
    /// Creates a new [`RxMessage`].
    #[must_use]
    pub const fn new(data: u8, ack: Ack) -> Self {
        Self { data, ack }
    }
}

#[cfg(test)]
mod tests {
    use std::mem::offset_of;
    use std::mem::size_of;

    use super::*;

    #[test]
    fn rx_size() {
        assert_eq!(2, size_of::<RxMessage>());
        assert_eq!(0, offset_of!(RxMessage, data));
        assert_eq!(1, offset_of!(RxMessage, ack));
    }
}
