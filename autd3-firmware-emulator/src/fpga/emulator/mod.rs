mod gpio_out;
mod memory;
mod modulation;
mod output_mask;
mod phase_corr;
mod pwe;
mod silencer;
mod stm;
mod swapchain;

use autd3_core::firmware::Segment;
use autd3_driver::ethercat::DcSysTime;

use getset::{Getters, MutGetters};
use memory::Memory;

use super::params::{
    ADDR_CTL_FLAG, ADDR_FPGA_STATE, CTL_FLAG_FORCE_FAN_BIT, CTL_FLAG_MOD_SET_BIT,
    CTL_FLAG_STM_SET_BIT,
};

pub use silencer::SilencerEmulator;

const CTL_FLAG_MOD_SET: u16 = 1 << CTL_FLAG_MOD_SET_BIT;
const CTL_FLAG_STM_SET: u16 = 1 << CTL_FLAG_STM_SET_BIT;

#[derive(Getters, MutGetters)]
pub struct FPGAEmulator {
    #[getset(get = "pub", get_mut = "pub")]
    pub(crate) mem: Memory,
    mod_swapchain: swapchain::Swapchain<CTL_FLAG_MOD_SET>,
    stm_swapchain: swapchain::Swapchain<CTL_FLAG_STM_SET>,
}

impl FPGAEmulator {
    #[must_use]
    pub(crate) fn new(num_transducers: usize) -> Self {
        let mut fpga = Self {
            mem: Memory::new(num_transducers),
            mod_swapchain: swapchain::Swapchain::new(),
            stm_swapchain: swapchain::Swapchain::new(),
        };
        fpga.init();
        fpga
    }

    pub(crate) fn init(&mut self) {
        self.mod_swapchain.init();
        self.stm_swapchain.init();
    }

    pub(crate) fn write(&mut self, addr: u16, data: u16) {
        self.mem.write(addr, data);
    }

    pub(crate) fn set_and_wait_update(&mut self, sys_time: DcSysTime) {
        let addr = ((crate::fpga::params::BRAM_SELECT_CONTROLLER as u16 & 0x0003) << 14)
            | (crate::fpga::params::ADDR_CTL_FLAG as u16 & 0x3FFF);
        if (self.read(addr) & CTL_FLAG_MOD_SET) == CTL_FLAG_MOD_SET {
            self.mod_swapchain.set(
                sys_time,
                self.modulation_loop_count(self.req_modulation_segment()),
                self.modulation_freq_divide(self.req_modulation_segment()),
                self.modulation_cycle(self.req_modulation_segment()),
                self.req_modulation_segment(),
                self.modulation_transition_mode(),
            );
        }
        if (self.read(addr) & CTL_FLAG_STM_SET) == CTL_FLAG_STM_SET {
            self.stm_swapchain.set(
                sys_time,
                self.stm_loop_count(self.req_stm_segment()),
                self.stm_freq_divide(self.req_stm_segment()),
                self.stm_cycle(self.req_stm_segment()),
                self.req_stm_segment(),
                self.stm_transition_mode(),
            );
        }
    }

    // GRCOV_EXCL_START
    pub fn update(&mut self) {
        self.update_with_sys_time(DcSysTime::now());
    }
    // GRCOV_EXCL_STOP

    pub fn update_with_sys_time(&mut self, sys_time: DcSysTime) {
        self.mod_swapchain.update(self.gpio_in(), sys_time);
        self.stm_swapchain.update(self.gpio_in(), sys_time);

        let mut fpga_state = self.fpga_state();
        match self.current_mod_segment() {
            Segment::S0 => fpga_state &= !(1 << 1),
            Segment::S1 => fpga_state |= 1 << 1,
        }
        match self.current_stm_segment() {
            Segment::S0 => fpga_state &= !(1 << 2),
            Segment::S1 => fpga_state |= 1 << 2,
        }
        if self.stm_cycle(self.current_stm_segment()) == 1 {
            fpga_state |= 1 << 3;
        } else {
            fpga_state &= !(1 << 3);
        }
        self.mem.update(fpga_state);
    }

    #[must_use]
    pub fn fpga_state(&self) -> u16 {
        self.mem.controller_bram.read(ADDR_FPGA_STATE)
    }

    pub fn assert_thermal_sensor(&mut self) {
        let state = self.mem.controller_bram.read(ADDR_FPGA_STATE);
        self.mem
            .controller_bram
            .write(ADDR_FPGA_STATE, state | (1 << 0));
    }

    pub fn deassert_thermal_sensor(&mut self) {
        let state = self.mem.controller_bram.read(ADDR_FPGA_STATE);
        self.mem
            .controller_bram
            .write(ADDR_FPGA_STATE, state & !(1 << 0));
    }

    #[must_use]
    pub fn is_thermo_asserted(&self) -> bool {
        (self.mem.controller_bram.read(ADDR_FPGA_STATE) & (1 << 0)) != 0
    }

    #[must_use]
    pub fn is_force_fan(&self) -> bool {
        (self.mem.controller_bram.read(ADDR_CTL_FLAG) & (1 << CTL_FLAG_FORCE_FAN_BIT)) != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thermo() {
        let mut fpga = FPGAEmulator::new(249);
        assert!(!fpga.is_thermo_asserted());
        fpga.assert_thermal_sensor();
        assert!(fpga.is_thermo_asserted());
        fpga.deassert_thermal_sensor();
        assert!(!fpga.is_thermo_asserted());
    }
}
