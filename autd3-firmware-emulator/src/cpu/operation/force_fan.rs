use crate::{cpu::params::*, CPUEmulator};

#[repr(C, align(2))]
struct ConfigureForceFan {
    tag: u8,
    value: u8,
}

impl CPUEmulator {
    pub(crate) fn configure_force_fan(&mut self, data: &[u8]) -> u8 {
        let d = Self::cast::<ConfigureForceFan>(data);
        if d.value != 0x00 {
            self.fpga_flags_internal |= CTL_FLAG_FORCE_FAN;
        } else {
            self.fpga_flags_internal &= !CTL_FLAG_FORCE_FAN;
        }

        NO_ERR
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configure_force_fan_memory_layout() {
        assert_eq!(2, std::mem::size_of::<ConfigureForceFan>());
        assert_eq!(0, std::mem::offset_of!(ConfigureForceFan, tag));
        assert_eq!(1, std::mem::offset_of!(ConfigureForceFan, value));
    }
}
