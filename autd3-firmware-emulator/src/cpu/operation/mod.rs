use crate::CPUEmulator;

use super::params::{
    ADDR_CTL_FLAG, BRAM_SELECT_CONTROLLER, TRANSITION_MODE_EXT, TRANSITION_MODE_GPIO,
    TRANSITION_MODE_IMMEDIATE, TRANSITION_MODE_NONE, TRANSITION_MODE_SYNC_IDX,
    TRANSITION_MODE_SYS_TIME,
};

mod clear;
#[cfg(feature = "dynamic_freq")]
mod clock;
mod cpu_gpio_out;
mod debug;
mod force_fan;
mod gain;
mod gpio_in;
mod info;
mod modulation;
mod phase_corr;
mod pulse_width_encoder;
mod reads_fpga_state;
mod silecer;
mod stm;
mod sync;

impl CPUEmulator {
    pub(crate) fn validate_transition_mode(
        current_segment: u8,
        segment: u8,
        rep: u16,
        mode: u8,
    ) -> bool {
        if mode == TRANSITION_MODE_NONE {
            return false;
        }
        if current_segment == segment {
            return mode == TRANSITION_MODE_SYNC_IDX
                || mode == TRANSITION_MODE_SYS_TIME
                || mode == TRANSITION_MODE_GPIO;
        }
        match rep {
            0xFFFF => {
                mode == TRANSITION_MODE_SYNC_IDX
                    || mode == TRANSITION_MODE_SYS_TIME
                    || mode == TRANSITION_MODE_GPIO
            }
            _ => mode == TRANSITION_MODE_IMMEDIATE || mode == TRANSITION_MODE_EXT,
        }
    }

    pub(crate) fn set_and_wait_update(&mut self, flag: u16) {
        self.bram_write(
            BRAM_SELECT_CONTROLLER,
            ADDR_CTL_FLAG,
            self.fpga_flags_internal | flag,
        );
        self.fpga.set_and_wait_update(self.dc_sys_time);
        self.bram_write(
            BRAM_SELECT_CONTROLLER,
            ADDR_CTL_FLAG,
            self.fpga_flags_internal,
        );
    }
}
