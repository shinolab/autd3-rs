use crate::{cpu::params::*, CPUEmulator};

#[repr(C, align(2))]
struct Clear {
    tag: u8,
}

impl CPUEmulator {
    pub(crate) unsafe fn clear(&mut self, data: &[u8]) -> u8 {
        let _d = Self::cast::<Clear>(data);

        self.read_fpga_state = false;

        self.fpga_flags_internal = 0x0000;

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
            BRAM_ADDR_SILENCER_MODE,
            SILNCER_MODE_FIXED_COMPLETION_STEPS,
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

        self.mod_freq_div = [5120, 5120];
        self.mod_cycle = 2;
        self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_MOD_REQ_RD_SEGMENT, 0);
        self.bram_write(
            BRAM_SELECT_CONTROLLER,
            BRAM_ADDR_MOD_CYCLE_0,
            (self.mod_cycle.max(1) - 1) as _,
        );
        self.bram_cpy(
            BRAM_SELECT_CONTROLLER,
            BRAM_ADDR_MOD_FREQ_DIV_0_0,
            &self.mod_freq_div as *const _ as _,
            std::mem::size_of::<u32>() >> 1,
        );
        self.bram_write(
            BRAM_SELECT_CONTROLLER,
            BRAM_ADDR_MOD_CYCLE_1,
            (self.mod_cycle.max(1) - 1) as _,
        );
        self.bram_cpy(
            BRAM_SELECT_CONTROLLER,
            BRAM_ADDR_MOD_FREQ_DIV_1_0,
            &self.mod_freq_div as *const _ as _,
            std::mem::size_of::<u32>() >> 1,
        );
        self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_MOD_REP_0_0, 0xFFFF);
        self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_MOD_REP_0_1, 0xFFFF);
        self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_MOD_REP_1_0, 0xFFFF);
        self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_MOD_REP_1_1, 0xFFFF);
        self.change_mod_wr_segment(0);
        self.bram_write(BRAM_SELECT_MOD, 0, 0xFFFF);
        self.change_mod_wr_segment(1);
        self.bram_write(BRAM_SELECT_MOD, 0, 0xFFFF);

        self.stm_cycle = [1, 1];
        self.stm_mode = [STM_MODE_GAIN, STM_MODE_GAIN];
        self.stm_freq_div = [0xFFFFFFFF, 0xFFFFFFFF];
        self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_STM_MODE_0, STM_MODE_GAIN);
        self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_STM_MODE_1, STM_MODE_GAIN);
        self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_STM_REQ_RD_SEGMENT, 0);
        self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_STM_CYCLE_0, 0);
        self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_STM_FREQ_DIV_0_0, 0xFFFF);
        self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_STM_FREQ_DIV_0_1, 0xFFFF);
        self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_STM_CYCLE_1, 0);
        self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_STM_FREQ_DIV_1_0, 0xFFFF);
        self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_STM_FREQ_DIV_1_1, 0xFFFF);
        self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_STM_REP_0_0, 0xFFFF);
        self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_STM_REP_0_1, 0xFFFF);
        self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_STM_REP_1_0, 0xFFFF);
        self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_STM_REP_1_1, 0xFFFF);
        self.change_stm_wr_segment(0);
        self.change_stm_wr_page(0);
        self.bram_set(BRAM_SELECT_STM, 0, 0x0000, TRANS_NUM << 1);
        self.change_stm_wr_segment(1);
        self.change_stm_wr_page(0);
        self.bram_set(BRAM_SELECT_STM, 0, 0x0000, TRANS_NUM << 1);

        self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_DEBUG_OUT_IDX, 0xFF);

        NO_ERR
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clear_memory_layout() {
        assert_eq!(2, std::mem::size_of::<Clear>());
        assert_eq!(0, memoffset::offset_of!(Clear, tag));
    }
}
