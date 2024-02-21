#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GainSTMMode {
    PhaseIntensityFull = 0,
    PhaseFull = 1,
    PhaseHalf = 2,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gain_stm_mode() {
        assert_eq!(std::mem::size_of::<GainSTMMode>(), 1);

        assert_eq!(GainSTMMode::PhaseIntensityFull as u16, 0);
        assert_eq!(GainSTMMode::PhaseFull as u16, 1);
        assert_eq!(GainSTMMode::PhaseHalf as u16, 2);

        let mode = GainSTMMode::PhaseIntensityFull;

        let modec = Clone::clone(&mode);
        assert_eq!(modec, mode);
        assert_eq!(
            format!("{:?}", GainSTMMode::PhaseIntensityFull),
            "PhaseIntensityFull"
        );
        assert_eq!(format!("{:?}", GainSTMMode::PhaseFull), "PhaseFull");
        assert_eq!(format!("{:?}", GainSTMMode::PhaseHalf), "PhaseHalf");
    }
}
