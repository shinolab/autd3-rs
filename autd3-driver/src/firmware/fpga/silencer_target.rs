#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SilencerTarget {
    Intensity = 0,
    PulseWidth = 1,
}
