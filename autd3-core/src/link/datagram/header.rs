use zerocopy::{FromZeros, Immutable, IntoBytes};

use super::msg_id::MsgId;

#[doc(hidden)]
#[repr(C, align(2))]
#[derive(Clone, Debug, PartialEq, Eq, IntoBytes, Immutable, FromZeros)]
pub struct Header {
    pub msg_id: MsgId,
    __: u8,
    pub slot_2_offset: u16,
}

#[cfg(test)]
mod tests {
    use core::mem::offset_of;
    use core::mem::size_of;

    use super::*;

    #[test]
    fn test_size() {
        assert_eq!(4, size_of::<Header>());
        assert_eq!(0, offset_of!(Header, msg_id));
        assert_eq!(2, offset_of!(Header, slot_2_offset));
    }
}
