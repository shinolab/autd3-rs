/*
 * File: clear.rs
 * Project: operation
 * Created Date: 30/12/2023
 * Author: Shun Suzuki
 * -----
 * Last Modified: 30/12/2023
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2023 Shun Suzuki. All rights reserved.
 *
 */

use crate::{
    cpu::params::{
        BRAM_ADDR_CTL_REG, BRAM_ADDR_DEBUG_OUT_IDX, BRAM_ADDR_MOD_ADDR_OFFSET, BRAM_ADDR_MOD_CYCLE,
        BRAM_ADDR_MOD_DELAY_BASE, BRAM_ADDR_MOD_FREQ_DIV_0,
        BRAM_ADDR_SILENCER_COMPLETION_STEPS_INTENSITY, BRAM_ADDR_SILENCER_COMPLETION_STEPS_PHASE,
        BRAM_ADDR_SILENCER_CTL_FLAG, BRAM_ADDR_SILENCER_UPDATE_RATE_INTENSITY,
        BRAM_ADDR_SILENCER_UPDATE_RATE_PHASE, BRAM_SELECT_CONTROLLER, BRAM_SELECT_MOD,
        BRAM_SELECT_NORMAL, ERR_NONE,
    },
    CPUEmulator,
};

use super::silecer::SILENCER_CTL_FLAG_FIXED_COMPLETION_STEPS;

#[repr(C, align(2))]
struct Clear {
    tag: u8,
}

impl CPUEmulator {
    pub(crate) fn clear(&mut self, data: &[u8]) -> u8 {
        let _d = Self::cast::<Clear>(data);

        self.mod_freq_div = 5120;
        self.stm_freq_div = 0xFFFFFFFF;

        self.read_fpga_info = false;

        self.fpga_flags_internal = 0x0000;
        self.bram_write(
            BRAM_SELECT_CONTROLLER,
            BRAM_ADDR_CTL_REG,
            self.fpga_flags_internal,
        );

        self.bram_write(
            BRAM_SELECT_CONTROLLER,
            BRAM_ADDR_SILENCER_UPDATE_RATE_INTENSITY,
            256,
        );
        self.bram_write(
            BRAM_SELECT_CONTROLLER,
            BRAM_ADDR_SILENCER_UPDATE_RATE_PHASE,
            256,
        );
        self.bram_write(
            BRAM_SELECT_CONTROLLER,
            BRAM_ADDR_SILENCER_CTL_FLAG,
            SILENCER_CTL_FLAG_FIXED_COMPLETION_STEPS,
        );
        self.bram_write(
            BRAM_SELECT_CONTROLLER,
            BRAM_ADDR_SILENCER_COMPLETION_STEPS_INTENSITY,
            10,
        );
        self.bram_write(
            BRAM_SELECT_CONTROLLER,
            BRAM_ADDR_SILENCER_COMPLETION_STEPS_PHASE,
            40,
        );
        self.silencer_strict_mode = true;
        self.min_freq_div_intensity = 10 << 9;
        self.min_freq_div_phase = 40 << 9;

        self.stm_cycle = 0;

        self.mod_cycle = 2;
        self.bram_write(
            BRAM_SELECT_CONTROLLER,
            BRAM_ADDR_MOD_CYCLE,
            (self.mod_cycle.max(1) - 1) as _,
        );
        self.bram_cpy(
            BRAM_SELECT_CONTROLLER,
            BRAM_ADDR_MOD_FREQ_DIV_0,
            &self.mod_freq_div as *const _ as _,
            std::mem::size_of::<u32>() >> 1,
        );
        self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_MOD_ADDR_OFFSET, 0x0000);
        self.bram_write(BRAM_SELECT_MOD, 0, 0xFFFF);

        self.bram_set(BRAM_SELECT_NORMAL, 0, 0x0000, self.num_transducers << 1);

        self.bram_set(
            BRAM_SELECT_CONTROLLER,
            BRAM_ADDR_MOD_DELAY_BASE,
            0x0000,
            self.num_transducers,
        );

        self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_DEBUG_OUT_IDX, 0xFF);

        ERR_NONE
    }
}
