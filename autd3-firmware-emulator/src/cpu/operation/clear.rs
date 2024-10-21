use autd3_driver::firmware::fpga::PWE_BUF_SIZE;

use crate::{cpu::params::*, CPUEmulator};

static ASIN_TABLE: &'static [u8; PWE_BUF_SIZE] = {
    #[repr(C)]
    pub struct AlignedBytes {
        pub __: [u16; 0],
        pub bytes: [u8; PWE_BUF_SIZE],
    }
    static ALIGNED: &AlignedBytes = &AlignedBytes {
        __: [],
        bytes: *include_bytes!("asin.dat"),
    };
    &ALIGNED.bytes
};

#[allow(dead_code)]
#[repr(C, align(2))]
struct Clear {
    tag: u8,
}

impl CPUEmulator {
    pub(crate) unsafe fn clear(&mut self, _data: &[u8]) -> u8 {
        // let _d = Self::cast::<Clear>(data);

        self.port_a_podr = 0x00;

        self.reads_fpga_state = false;

        self.fpga_flags_internal = 0x0000;

        self.bram_write(
            BRAM_SELECT_CONTROLLER,
            ADDR_SILENCER_UPDATE_RATE_INTENSITY,
            256,
        );
        self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_SILENCER_UPDATE_RATE_PHASE, 256);
        self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_SILENCER_FLAG, 0);
        self.bram_write(
            BRAM_SELECT_CONTROLLER,
            ADDR_SILENCER_COMPLETION_STEPS_INTENSITY,
            10,
        );
        self.bram_write(
            BRAM_SELECT_CONTROLLER,
            ADDR_SILENCER_COMPLETION_STEPS_PHASE,
            40,
        );
        self.silencer_strict_mode = true;
        self.min_freq_div_intensity = 10;
        self.min_freq_div_phase = 40;

        self.mod_freq_div = [0xFFFF, 0xFFFF];
        self.mod_rep = [0xFFFF, 0xFFFF];
        self.mod_cycle = 2;
        self.mod_segment = 0;
        self.bram_write(
            BRAM_SELECT_CONTROLLER,
            ADDR_MOD_TRANSITION_MODE,
            TRANSITION_MODE_SYNC_IDX as _,
        );
        self.bram_set(
            BRAM_SELECT_CONTROLLER,
            ADDR_MOD_TRANSITION_VALUE_0,
            0,
            std::mem::size_of::<u64>() >> 1,
        );
        self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_MOD_REQ_RD_SEGMENT, 0);
        self.bram_write(
            BRAM_SELECT_CONTROLLER,
            ADDR_MOD_CYCLE0,
            (self.mod_cycle.max(1) - 1) as _,
        );
        self.bram_write(
            BRAM_SELECT_CONTROLLER,
            ADDR_MOD_FREQ_DIV0,
            self.mod_freq_div[0],
        );
        self.bram_write(
            BRAM_SELECT_CONTROLLER,
            ADDR_MOD_CYCLE1,
            (self.mod_cycle.max(1) - 1) as _,
        );
        self.bram_write(
            BRAM_SELECT_CONTROLLER,
            ADDR_MOD_FREQ_DIV1,
            self.mod_freq_div[1],
        );
        self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_MOD_REP0, 0xFFFF);
        self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_MOD_REP1, 0xFFFF);
        self.change_mod_wr_segment(0);
        self.bram_write(BRAM_SELECT_MOD, 0, 0xFFFF);
        self.change_mod_wr_segment(1);
        self.bram_write(BRAM_SELECT_MOD, 0, 0xFFFF);

        self.stm_cycle = [1, 1];
        self.stm_mode = [STM_MODE_GAIN, STM_MODE_GAIN];
        self.stm_freq_div = [0xFFFF, 0xFFFF];
        self.stm_rep = [0xFFFF, 0xFFFF];
        self.stm_segment = 0;
        self.bram_write(
            BRAM_SELECT_CONTROLLER,
            ADDR_STM_TRANSITION_MODE,
            TRANSITION_MODE_SYNC_IDX as _,
        );
        self.bram_set(
            BRAM_SELECT_CONTROLLER,
            ADDR_STM_TRANSITION_VALUE_0,
            0,
            std::mem::size_of::<u64>() >> 1,
        );
        self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_STM_MODE0, STM_MODE_GAIN);
        self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_STM_MODE1, STM_MODE_GAIN);
        self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_STM_REQ_RD_SEGMENT, 0);
        self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_STM_CYCLE0, 0);
        self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_STM_FREQ_DIV0, 0xFFFF);
        self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_STM_CYCLE1, 0);
        self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_STM_FREQ_DIV1, 0xFFFF);
        self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_STM_REP0, 0xFFFF);
        self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_STM_REP1, 0xFFFF);
        self.change_stm_wr_segment(0);
        self.change_stm_wr_page(0);
        self.bram_set(BRAM_SELECT_STM, 0, 0x0000, TRANS_NUM);
        self.change_stm_wr_segment(1);
        self.change_stm_wr_page(0);
        self.bram_set(BRAM_SELECT_STM, 0, 0x0000, TRANS_NUM);

        self.bram_set(
            BRAM_SELECT_CONTROLLER,
            (BRAM_CNT_SEL_PHASE_CORR as u16) << 8,
            0,
            (TRANS_NUM + 1) >> 1,
        );

        self.bram_cpy(
            BRAM_SELECT_PWE_TABLE,
            0,
            ASIN_TABLE.as_ptr() as *const _,
            PWE_BUF_SIZE >> 1,
        );

        self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_DEBUG_VALUE0_0, 0x0000);
        self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_DEBUG_VALUE0_1, 0x0000);
        self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_DEBUG_VALUE0_2, 0x0000);
        self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_DEBUG_VALUE0_3, 0x0000);
        self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_DEBUG_VALUE1_0, 0x0000);
        self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_DEBUG_VALUE1_1, 0x0000);
        self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_DEBUG_VALUE1_2, 0x0000);
        self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_DEBUG_VALUE1_3, 0x0000);
        self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_DEBUG_VALUE2_0, 0x0000);
        self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_DEBUG_VALUE2_1, 0x0000);
        self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_DEBUG_VALUE2_2, 0x0000);
        self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_DEBUG_VALUE2_3, 0x0000);
        self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_DEBUG_VALUE3_0, 0x0000);
        self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_DEBUG_VALUE3_1, 0x0000);
        self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_DEBUG_VALUE3_2, 0x0000);
        self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_DEBUG_VALUE3_3, 0x0000);

        self.set_and_wait_update(CTL_FLAG_MOD_SET);
        self.set_and_wait_update(CTL_FLAG_STM_SET);
        self.set_and_wait_update(CTL_FLAG_SILENCER_SET);
        self.set_and_wait_update(CTL_FLAG_DEBUG_SET);

        NO_ERR
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg_attr(miri, ignore)]
    fn clear_memory_layout() {
        assert_eq!(2, std::mem::size_of::<Clear>());
        assert_eq!(0, std::mem::offset_of!(Clear, tag));
    }
}
