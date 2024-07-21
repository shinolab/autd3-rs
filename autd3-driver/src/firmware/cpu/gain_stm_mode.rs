#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum GainSTMMode {
    PhaseIntensityFull = 0,
    PhaseFull = 1,
    PhaseHalf = 2,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_size() {
        assert_eq!(1, std::mem::size_of::<GainSTMMode>());
    }
}
