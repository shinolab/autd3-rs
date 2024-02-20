use crate::common::Segment;

/// FPGA state
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FPGAState {
    pub(crate) state: u8,
}

impl FPGAState {
    /// Check if thermal sensor is asserted
    pub const fn is_thermal_assert(&self) -> bool {
        (self.state & (1 << 0)) != 0
    }

    /// Current mod segment
    pub const fn current_mod_segment(&self) -> Segment {
        match self.state & (1 << 1) {
            0 => Segment::S0,
            _ => Segment::S1,
        }
    }

    /// Current stm segment
    pub const fn current_stm_segment(&self) -> Option<Segment> {
        if !self.is_stm_mode() {
            return None;
        }
        match self.state & (1 << 2) {
            0 => Some(Segment::S0),
            _ => Some(Segment::S1),
        }
    }

    /// Current gain segment
    pub const fn current_gain_segment(&self) -> Option<Segment> {
        if !self.is_gain_mode() {
            return None;
        }
        match self.state & (1 << 2) {
            0 => Some(Segment::S0),
            _ => Some(Segment::S1),
        }
    }

    /// Check if gain mode
    pub const fn is_gain_mode(&self) -> bool {
        (self.state & (1 << 3)) != 0
    }

    /// Check if stm mode
    pub const fn is_stm_mode(&self) -> bool {
        !self.is_gain_mode()
    }

    pub const fn state(&self) -> u8 {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::*;

    #[test]
    fn fpga_state() {
        assert_eq!(size_of::<FPGAState>(), 1);

        let info = FPGAState { state: 0x00 };
        assert!(!info.is_thermal_assert());
        assert_eq!(Segment::S0, info.current_mod_segment());
        assert_eq!(Some(Segment::S0), info.current_stm_segment());
        assert_eq!(None, info.current_gain_segment());
        assert_eq!(info.state(), 0x00);

        let info = FPGAState { state: 0x09 };
        assert!(info.is_thermal_assert());
        assert_eq!(None, info.current_stm_segment());
        assert_eq!(Some(Segment::S0), info.current_gain_segment());
        assert_eq!(info.state(), 0x09);
    }

    #[test]
    fn fpga_state_derive() {
        let info = FPGAState { state: 0x00 };
        let info2 = info;

        assert_eq!(info, info2);
        assert_eq!(format!("{:?}", info), "FPGAState { state: 0 }");
    }
}
