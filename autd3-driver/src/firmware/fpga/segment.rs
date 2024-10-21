#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Segment {
    #[default]
    S0 = 0,
    S1 = 1,
}
