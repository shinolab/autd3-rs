use autd3_driver::firmware::latest::fpga::PWE_BUF_SIZE;

use crate::{CPUEmulator, cpu::params::*};

static ASIN_TABLE: &[u8; PWE_BUF_SIZE] = &[
    0x00, 0x01, 0x01, 0x02, 0x03, 0x03, 0x04, 0x04, 0x05, 0x06, 0x06, 0x07, 0x08, 0x08, 0x09, 0x0a,
    0x0a, 0x0b, 0x0c, 0x0c, 0x0d, 0x0d, 0x0e, 0x0f, 0x0f, 0x10, 0x11, 0x11, 0x12, 0x13, 0x13, 0x14,
    0x15, 0x15, 0x16, 0x16, 0x17, 0x18, 0x18, 0x19, 0x1a, 0x1a, 0x1b, 0x1c, 0x1c, 0x1d, 0x1e, 0x1e,
    0x1f, 0x20, 0x20, 0x21, 0x21, 0x22, 0x23, 0x23, 0x24, 0x25, 0x25, 0x26, 0x27, 0x27, 0x28, 0x29,
    0x29, 0x2a, 0x2b, 0x2b, 0x2c, 0x2d, 0x2d, 0x2e, 0x2f, 0x2f, 0x30, 0x31, 0x31, 0x32, 0x33, 0x33,
    0x34, 0x35, 0x35, 0x36, 0x37, 0x37, 0x38, 0x39, 0x39, 0x3a, 0x3b, 0x3b, 0x3c, 0x3d, 0x3e, 0x3e,
    0x3f, 0x40, 0x40, 0x41, 0x42, 0x42, 0x43, 0x44, 0x44, 0x45, 0x46, 0x47, 0x47, 0x48, 0x49, 0x49,
    0x4a, 0x4b, 0x4c, 0x4c, 0x4d, 0x4e, 0x4e, 0x4f, 0x50, 0x51, 0x51, 0x52, 0x53, 0x53, 0x54, 0x55,
    0x56, 0x56, 0x57, 0x58, 0x59, 0x59, 0x5a, 0x5b, 0x5c, 0x5c, 0x5d, 0x5e, 0x5f, 0x5f, 0x60, 0x61,
    0x62, 0x63, 0x63, 0x64, 0x65, 0x66, 0x66, 0x67, 0x68, 0x69, 0x6a, 0x6a, 0x6b, 0x6c, 0x6d, 0x6e,
    0x6f, 0x6f, 0x70, 0x71, 0x72, 0x73, 0x74, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7a, 0x7a, 0x7b,
    0x7c, 0x7d, 0x7e, 0x7f, 0x80, 0x81, 0x82, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8a,
    0x8b, 0x8c, 0x8d, 0x8e, 0x8f, 0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9a,
    0x9b, 0x9d, 0x9e, 0x9f, 0xa0, 0xa1, 0xa2, 0xa3, 0xa5, 0xa6, 0xa7, 0xa8, 0xaa, 0xab, 0xac, 0xad,
    0xaf, 0xb0, 0xb2, 0xb3, 0xb4, 0xb6, 0xb7, 0xb9, 0xba, 0xbc, 0xbd, 0xbf, 0xc1, 0xc2, 0xc4, 0xc6,
    0xc8, 0xca, 0xcc, 0xce, 0xd0, 0xd2, 0xd5, 0xd7, 0xda, 0xdd, 0xe0, 0xe3, 0xe7, 0xec, 0xf2, 0x00,
];

#[allow(dead_code)]
#[repr(C, align(2))]
struct Clear {
    tag: u8,
}

impl CPUEmulator {
    #[must_use]
    pub(crate) unsafe fn clear(&mut self, _data: &[u8]) -> u8 {
        unsafe {
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
            self.silencer_strict = true;
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

            (0..PWE_BUF_SIZE).for_each(|i| {
                self.bram_write(BRAM_SELECT_PWE_TABLE, i as _, ASIN_TABLE[i] as u16);
            });
            self.bram_write(BRAM_SELECT_PWE_TABLE, PWE_BUF_SIZE as u16 - 1, 0x100);

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clear_memory_layout() {
        assert_eq!(2, std::mem::size_of::<Clear>());
        assert_eq!(0, std::mem::offset_of!(Clear, tag));
    }
}
