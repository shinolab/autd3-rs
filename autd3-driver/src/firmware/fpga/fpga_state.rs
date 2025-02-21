use crate::firmware::fpga::Segment;

use autd3_core::link::RxMessage;
use getset::CopyGetters;

const THERMAL_ASSERT_BIT: u8 = 1 << 0;
const CURRENT_MOD_SEGMENT_BIT: u8 = 1 << 1;
const CURRENT_STM_SEGMENT_BIT: u8 = 1 << 2;
const CURRENT_GAIN_SEGMENT_BIT: u8 = 1 << 2;
const IS_GAIN_MODE_BIT: u8 = 1 << 3;
const READS_FPGA_STATE_ENABLED: u8 = 1 << 7;

/// FPGA state.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, CopyGetters)]
pub struct FPGAState {
    #[doc(hidden)]
    #[getset(get_copy = "pub")]
    pub(crate) state: u8,
}

impl FPGAState {
    /// `true` if the thermal sensor is asserted.
    #[must_use]
    pub const fn is_thermal_assert(&self) -> bool {
        (self.state & THERMAL_ASSERT_BIT) != 0
    }

    /// The current Modulation segment.
    #[must_use]
    pub const fn current_mod_segment(&self) -> Segment {
        match self.state & CURRENT_MOD_SEGMENT_BIT {
            0 => Segment::S0,
            _ => Segment::S1,
        }
    }

    /// The current STM segment. `None` if the current mode is not STM.
    #[must_use]
    pub const fn current_stm_segment(&self) -> Option<Segment> {
        if !self.is_stm_mode() {
            return None;
        }
        match self.state & CURRENT_STM_SEGMENT_BIT {
            0 => Some(Segment::S0),
            _ => Some(Segment::S1),
        }
    }

    /// The current Gain segment. `None` if the current mode is not Gain.
    #[must_use]
    pub const fn current_gain_segment(&self) -> Option<Segment> {
        if !self.is_gain_mode() {
            return None;
        }
        match self.state & CURRENT_GAIN_SEGMENT_BIT {
            0 => Some(Segment::S0),
            _ => Some(Segment::S1),
        }
    }

    /// `true` if the current mode is Gain.
    #[must_use]
    pub const fn is_gain_mode(&self) -> bool {
        (self.state & IS_GAIN_MODE_BIT) != 0
    }

    /// `true` if the current mode is STM.
    #[must_use]
    pub const fn is_stm_mode(&self) -> bool {
        !self.is_gain_mode()
    }

    #[doc(hidden)]
    #[must_use]
    pub fn from_rx(msg: &RxMessage) -> Option<Self> {
        if msg.data() & READS_FPGA_STATE_ENABLED != 0 {
            Some(FPGAState { state: msg.data() })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::*;

    #[test]
    fn size() {
        assert_eq!(1, size_of::<FPGAState>());
        assert_eq!(0, std::mem::offset_of!(FPGAState, state));
    }

    #[rstest::rstest]
    #[test]
    #[case(false, 0b0)]
    #[case(true, 0b1)]
    fn is_thermal_assert(#[case] expected: bool, #[case] state: u8) {
        assert_eq!(expected, FPGAState { state }.is_thermal_assert());
    }

    #[rstest::rstest]
    #[test]
    #[case(Segment::S0, 0b00)]
    #[case(Segment::S1, 0b10)]
    fn current_mod_segment(#[case] expected: Segment, #[case] state: u8) {
        assert_eq!(expected, FPGAState { state }.current_mod_segment());
    }

    #[rstest::rstest]
    #[test]
    #[case(false, 0b0000)]
    #[case(true, 0b1000)]
    fn is_gain_mode(#[case] expected: bool, #[case] state: u8) {
        assert_eq!(expected, FPGAState { state }.is_gain_mode());
    }

    #[rstest::rstest]
    #[test]
    #[case(false, 0b1000)]
    #[case(true, 0b0000)]
    fn is_stm_mode(#[case] expected: bool, #[case] state: u8) {
        assert_eq!(expected, FPGAState { state }.is_stm_mode());
    }

    #[rstest::rstest]
    #[test]
    #[case(None, 0b1000)]
    #[case(Some(Segment::S0), 0b0000)]
    #[case(Some(Segment::S1), 0b0100)]
    fn current_stm_segment(#[case] expected: Option<Segment>, #[case] state: u8) {
        assert_eq!(expected, FPGAState { state }.current_stm_segment());
    }

    #[rstest::rstest]
    #[test]
    #[case(None, 0b0000)]
    #[case(Some(Segment::S0), 0b1000)]
    #[case(Some(Segment::S1), 0b1100)]
    fn current_gain_segment(#[case] expected: Option<Segment>, #[case] state: u8) {
        assert_eq!(expected, FPGAState { state }.current_gain_segment());
    }
}
