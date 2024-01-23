use crate::{
    cpu::params::{CTL_FLAG_FORCE_FAN, ERR_NONE},
    CPUEmulator,
};

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

        ERR_NONE
    }
}
