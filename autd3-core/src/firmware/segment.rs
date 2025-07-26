/// Segment of the FPGA memory
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Segment {
    /// Segment 0
    #[default]
    S0 = 0,
    /// Segment 1
    S1 = 1,
}
