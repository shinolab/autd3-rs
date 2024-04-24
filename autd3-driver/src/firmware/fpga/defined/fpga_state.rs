use crate::firmware::fpga::Segment;

const THERMAL_ASSERT_BIT: u8 = 1 << 0;
const CURRENT_MOD_SEGMENT_BIT: u8 = 1 << 1;
const CURRENT_STM_SEGMENT_BIT: u8 = 1 << 2;
const CURRENT_GAIN_SEGMENT_BIT: u8 = 1 << 2;
const IS_GAIN_MODE_BIT: u8 = 1 << 3;

/// FPGA state
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FPGAState {
    pub(crate) state: u8,
}

impl FPGAState {
    /// Check if thermal sensor is asserted
    pub const fn is_thermal_assert(&self) -> bool {
        (self.state & THERMAL_ASSERT_BIT) != 0
    }

    /// Current mod segment
    pub const fn current_mod_segment(&self) -> Segment {
        match self.state & CURRENT_MOD_SEGMENT_BIT {
            0 => Segment::S0,
            _ => Segment::S1,
        }
    }

    /// Current stm segment
    pub const fn current_stm_segment(&self) -> Option<Segment> {
        if !self.is_stm_mode() {
            return None;
        }
        match self.state & CURRENT_STM_SEGMENT_BIT {
            0 => Some(Segment::S0),
            _ => Some(Segment::S1),
        }
    }

    /// Current gain segment
    pub const fn current_gain_segment(&self) -> Option<Segment> {
        if !self.is_gain_mode() {
            return None;
        }
        match self.state & CURRENT_GAIN_SEGMENT_BIT {
            0 => Some(Segment::S0),
            _ => Some(Segment::S1),
        }
    }

    /// Check if gain mode
    pub const fn is_gain_mode(&self) -> bool {
        (self.state & IS_GAIN_MODE_BIT) != 0
    }

    /// Check if stm mode
    pub const fn is_stm_mode(&self) -> bool {
        !self.is_gain_mode()
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::*;

    #[test]
    fn test_size() {
        assert_eq!(1, size_of::<FPGAState>());
        assert_eq!(0, std::mem::offset_of!(FPGAState, state));
    }

    #[rstest::rstest]
    #[test]
    #[case(false, 0b0)]
    #[case(true, 0b1)]
    fn test_is_thermal_assert(#[case] expected: bool, #[case] state: u8) {
        assert_eq!(expected, FPGAState { state }.is_thermal_assert());
    }

    #[rstest::rstest]
    #[test]
    #[case(Segment::S0, 0b00)]
    #[case(Segment::S1, 0b10)]
    fn test_current_mod_segment(#[case] expected: Segment, #[case] state: u8) {
        assert_eq!(expected, FPGAState { state }.current_mod_segment());
    }

    #[rstest::rstest]
    #[test]
    #[case(false, 0b0000)]
    #[case(true, 0b1000)]
    fn test_is_gain_mode(#[case] expected: bool, #[case] state: u8) {
        assert_eq!(expected, FPGAState { state }.is_gain_mode());
    }

    #[rstest::rstest]
    #[test]
    #[case(false, 0b1000)]
    #[case(true, 0b0000)]
    fn test_is_stm_mode(#[case] expected: bool, #[case] state: u8) {
        assert_eq!(expected, FPGAState { state }.is_stm_mode());
    }

    #[rstest::rstest]
    #[test]
    #[case(None, 0b1000)]
    #[case(Some(Segment::S0), 0b0000)]
    #[case(Some(Segment::S1), 0b0100)]
    fn test_current_stm_segment(#[case] expected: Option<Segment>, #[case] state: u8) {
        assert_eq!(expected, FPGAState { state }.current_stm_segment());
    }

    #[rstest::rstest]
    #[test]
    #[case(None, 0b0000)]
    #[case(Some(Segment::S0), 0b1000)]
    #[case(Some(Segment::S1), 0b1100)]
    fn test_current_gain_segment(#[case] expected: Option<Segment>, #[case] state: u8) {
        assert_eq!(expected, FPGAState { state }.current_gain_segment());
    }
}
