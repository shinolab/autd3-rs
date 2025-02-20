use crate::{CPUEmulator, cpu::params::*};

mod foci;
mod gain;

impl CPUEmulator {
    pub(crate) unsafe fn stm_segment_update(&mut self, segment: u8, mode: u8, value: u64) -> u8 {
        self.bram_write(
            BRAM_SELECT_CONTROLLER,
            ADDR_STM_REQ_RD_SEGMENT,
            segment as _,
        );
        if (mode == TRANSITION_MODE_SYS_TIME)
            && (value < self.dc_sys_time.sys_time() + SYS_TIME_TRANSITION_MARGIN)
        {
            return ERR_MISS_TRANSITION_TIME;
        }
        self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_STM_TRANSITION_MODE, mode as _);
        self.bram_cpy(
            BRAM_SELECT_CONTROLLER,
            ADDR_STM_TRANSITION_VALUE_0,
            &raw const value as _,
            std::mem::size_of::<u64>() >> 1,
        );
        self.set_and_wait_update(CTL_FLAG_STM_SET);
        NO_ERR
    }

    pub(crate) unsafe fn change_stm_wr_segment(&mut self, segment: u16) {
        self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_STM_MEM_WR_SEGMENT, segment);
    }

    pub(crate) unsafe fn change_stm_wr_page(&mut self, page: u16) {
        self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_STM_MEM_WR_PAGE, page as _);
    }
}
