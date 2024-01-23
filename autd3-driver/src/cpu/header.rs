pub const MSG_ID_MAX: u8 = 0x7F;

#[repr(C)]
pub struct Header {
    pub msg_id: u8,
    _pad: u8,
    pub slot_2_offset: u16,
}
