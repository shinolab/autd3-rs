use derive_more::Display;
use derive_new::new;
use getset::CopyGetters;
use zerocopy::{FromBytes, Immutable, IntoBytes};

/// PDO input data representation
#[derive(
    Clone, Copy, PartialEq, Eq, Debug, new, CopyGetters, IntoBytes, Immutable, FromBytes, Display,
)]
#[display("{:?}", self)]
#[repr(C)]
pub struct RxMessage {
    #[getset(get_copy = "pub")]
    /// Received data
    data: u8,
    #[getset(get_copy = "pub")]
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
