#[derive(Clone, Copy)]
#[repr(C)]
pub struct GainControlFlags(u16);

bitflags::bitflags! {
    impl GainControlFlags : u16 {
        const NONE           = 0;
        const UPDATE_SEGMENT = 1 << 0;
    }
}
