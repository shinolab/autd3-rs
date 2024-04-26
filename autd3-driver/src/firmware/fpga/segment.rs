#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Segment {
    #[default]
    S0 = 0,
    S1 = 1,
}
