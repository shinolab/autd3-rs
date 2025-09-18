use zerocopy::{FromBytes, Immutable, IntoBytes};

/// Acknowledgement structure for received messages
#[bitfield_struct::bitfield(u8)]
#[derive(IntoBytes, Immutable, FromBytes, PartialEq, Eq)]
pub struct Ack {
    #[bits(4)]
    pub msg_id: u8,
    #[bits(4)]
    pub err: u8,
}

/// PDO input data representation
#[derive(Clone, Copy, PartialEq, Eq, Debug, IntoBytes, Immutable, FromBytes)]
#[repr(C)]
pub struct RxMessage {
    data: u8,
    ack: Ack,
}

impl RxMessage {
    /// Creates a new [`RxMessage`].
    #[must_use]
    pub const fn new(data: u8, ack: Ack) -> Self {
        Self { data, ack }
    }

    /// Returns the received data.
    #[must_use]
    pub const fn data(&self) -> u8 {
        self.data
    }

    /// Returns the acknowledgement.
    #[must_use]
    pub const fn ack(&self) -> Ack {
        self.ack
    }
}

#[cfg(test)]
mod tests {
    use core::mem::offset_of;
    use core::mem::size_of;

    use super::*;

    #[test]
    fn rx_size() {
        assert_eq!(2, size_of::<RxMessage>());
        assert_eq!(0, offset_of!(RxMessage, data));
        assert_eq!(1, offset_of!(RxMessage, ack));
    }
}
