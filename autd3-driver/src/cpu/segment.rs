#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Segment {
    S0 = 0,
    S1 = 1,
}

impl Default for Segment {
    fn default() -> Self {
        Segment::S0
    }
}
