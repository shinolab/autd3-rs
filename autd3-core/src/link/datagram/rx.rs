/// Acknowledgement structure for received messages
#[derive(PartialEq, Eq, Copy, Clone)]
pub struct Ack(u8);

impl Ack {
    #[doc(hidden)]
    #[must_use]
    pub const fn new(msg_id: u8, err: u8) -> Self {
        Self((err & 0x0F) << 4 | (msg_id & 0x0F))
    }

    #[must_use]
    /// Returns the message ID.
    pub const fn msg_id(&self) -> u8 {
        self.0 & 0x0F
    }

    #[must_use]
    /// Returns the error code.
    pub const fn err(&self) -> u8 {
        (self.0 >> 4) & 0x0F
    }
}

impl core::fmt::Debug for Ack {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Ack")
            .field("msg_id", &self.msg_id())
            .field("err", &self.err())
            .finish()
    }
}

/// PDO input data representation
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
    fn ack_size() {
        assert_eq!(1, size_of::<Ack>());
    }

    #[test]
    fn ack_bitfield() {
        let ack = Ack::new(0x05, 0x03);
        assert_eq!(5, ack.msg_id());
        assert_eq!(3, ack.err());
        assert_eq!(0x35, ack.0);
    }

    #[test]
    fn ack_debug() {
        let ack = Ack::new(0x05, 0x03);
        let debug = format!("{:?}", ack);
        assert_eq!("Ack { msg_id: 5, err: 3 }", debug);
    }

    #[test]
    fn rx_size() {
        assert_eq!(2, size_of::<RxMessage>());
        assert_eq!(0, offset_of!(RxMessage, data));
        assert_eq!(1, offset_of!(RxMessage, ack));
    }
}
