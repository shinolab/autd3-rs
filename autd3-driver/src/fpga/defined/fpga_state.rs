/// FPGA state
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FPGAState {
    pub(crate) state: u8,
}

impl FPGAState {
    /// Check if thermal sensor is asserted
    pub const fn is_thermal_assert(&self) -> bool {
        (self.state & 0x01) != 0
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
        assert_eq!(info.state(), 0x00);

        let info = FPGAState { state: 0x01 };
        assert!(info.is_thermal_assert());
        assert_eq!(info.state(), 0x01);
    }

    #[test]
    fn fpga_state_derive() {
        let info = FPGAState { state: 0x00 };
        let info2 = info.clone();

        assert_eq!(info, info2);
        assert_eq!(format!("{:?}", info), "FPGAState { state: 0 }");
    }
}
