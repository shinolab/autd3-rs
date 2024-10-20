use zerocopy::{Immutable, IntoBytes};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoBytes, Immutable)]
#[non_exhaustive]
pub enum GainSTMMode {
    PhaseIntensityFull = 0,
    PhaseFull = 1,
    PhaseHalf = 2,
}
