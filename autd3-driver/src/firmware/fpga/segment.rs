#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
#[non_exhaustive]
pub enum Segment {
    #[default]
    S0 = 0,
    S1 = 1,
}
