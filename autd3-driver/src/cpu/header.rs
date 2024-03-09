pub const MSG_ID_MAX: u8 = 0x7F;

#[repr(C)]
#[derive(Clone)]
pub struct Header {
    pub msg_id: u8,
    pub(crate) _pad: u8,
    pub slot_2_offset: u16,
}

#[cfg(test)]
mod tests {
    use memoffset::offset_of;
    use std::mem::size_of;

    use super::*;

    #[test]
    fn test_header_size() {
        assert_eq!(4, size_of::<Header>());
        assert_eq!(0, offset_of!(Header, msg_id));
        assert_eq!(2, offset_of!(Header, slot_2_offset));
    }
}
