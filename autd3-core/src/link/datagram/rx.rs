use autd3_derive::Builder;
use derive_more::Display;
use derive_new::new;
use zerocopy::{FromBytes, Immutable, IntoBytes};

/// PDO input data representation
#[derive(
    Clone, Copy, PartialEq, Eq, Debug, new, Builder, IntoBytes, Immutable, FromBytes, Display,
)]
#[display("{:?}", self)]
#[repr(C)]
pub struct RxMessage {
    #[get]
    /// Received data
    data: u8,
    #[get]
    /// Acknowledgement
    ack: u8,
}

#[cfg(test)]
mod tests {
    use std::mem::offset_of;
    use std::mem::size_of;

    use super::*;

    #[test]
    fn test_message_size() {
        assert_eq!(2, size_of::<RxMessage>());
        assert_eq!(0, offset_of!(RxMessage, data));
        assert_eq!(1, offset_of!(RxMessage, ack));
    }
}
