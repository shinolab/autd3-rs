use crate::{cpu::params::*, CPUEmulator};

mod focus;
mod gain;

impl CPUEmulator {
    pub(crate) unsafe fn change_stm_wr_segment(&mut self, segment: u16) {
        self.bram_write(
            BRAM_SELECT_CONTROLLER,
            BRAM_ADDR_STM_MEM_WR_SEGMENT,
            segment,
        );
    }

    pub(crate) unsafe fn change_stm_wr_page(&mut self, page: u16) {
        self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_STM_MEM_WR_PAGE, page as _);
    }
}
