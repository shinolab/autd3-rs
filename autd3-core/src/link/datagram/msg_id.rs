use zerocopy::{FromZeros, Immutable, IntoBytes};

#[doc(hidden)]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, IntoBytes, Immutable, FromZeros)]
pub struct MsgId(u8);

impl MsgId {
    pub const MAX: Self = MsgId(0x7F);

    pub const fn new(id: u8) -> Self {
        MsgId(id)
    }

    pub const fn get(&self) -> u8 {
        self.0
    }

    pub const fn increment(&mut self) {
        self.0 += 1;
        if self.0 > Self::MAX.0 {
            self.0 = 0;
        }
    }
}
