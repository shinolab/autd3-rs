use crate::{cpu::params::*, CPUEmulator};

#[repr(C, align(2))]
struct Gain {
    tag: u8,
    segment: u8,
    flag: u16,
}

impl CPUEmulator {
    pub(crate) unsafe fn write_gain(&mut self, data: &[u8]) -> u8 {
        let d = Self::cast::<Gain>(data);

        let segment = d.segment;

        let data = unsafe {
            std::slice::from_raw_parts(
                data[std::mem::size_of::<Gain>()..].as_ptr() as *const u16,
                (data.len() - 2) >> 1,
            )
        };

        match segment {
            0 => {
                self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_STM_FREQ_DIV_0_0, 0xFFFF);
                self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_STM_FREQ_DIV_0_1, 0xFFFF);
                self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_STM_REP_0_1, 0xFFFF);
                self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_STM_REP_0_1, 0xFFFF);
                self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_STM_CYCLE_0, 0);
                self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_STM_MODE_0, STM_MODE_GAIN);
            }
            1 => {
                self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_STM_FREQ_DIV_1_0, 0xFFFF);
                self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_STM_FREQ_DIV_1_1, 0xFFFF);
                self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_STM_REP_1_0, 0xFFFF);
                self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_STM_REP_1_1, 0xFFFF);
                self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_STM_CYCLE_1, 0);
                self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_STM_MODE_1, STM_MODE_GAIN);
            }
            _ => return ERR_INVALID_SEGMENT,
        }
        self.stm_freq_div[segment as usize] = 0xFFFFFFFF;

        self.change_stm_wr_segment(segment as _);
        self.change_stm_wr_page(0);
        (0..self.num_transducers).for_each(|i| self.bram_write(BRAM_SELECT_STM, i as _, data[i]));

        if (d.flag & GAIN_FLAG_UPDATE) == GAIN_FLAG_UPDATE {
            self.bram_write(
                BRAM_SELECT_CONTROLLER,
                BRAM_ADDR_STM_REQ_RD_SEGMENT,
                segment as _,
            );
        }

        NO_ERR
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gain_memory_layout() {
        assert_eq!(4, std::mem::size_of::<Gain>());
        assert_eq!(0, memoffset::offset_of!(Gain, tag));
        assert_eq!(1, memoffset::offset_of!(Gain, segment));
        assert_eq!(2, memoffset::offset_of!(Gain, flag));
    }
}
