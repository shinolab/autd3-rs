use crate::{CPUEmulator, cpu::params::*};

#[repr(C, align(2))]
struct Gain {
    tag: u8,
    segment: u8,
    flag: u8,
    __: u8,
}

#[repr(C, align(2))]
struct GainUpdate {
    tag: u8,
    segment: u8,
}

impl CPUEmulator {
    #[must_use]
    pub(crate) unsafe fn write_gain(&mut self, data: &[u8]) -> u8 {
        unsafe {
            let d = Self::cast::<Gain>(data);

            let segment = d.segment;
            self.stm_segment = segment;

            let data = std::slice::from_raw_parts(
                data[std::mem::size_of::<Gain>()..].as_ptr() as *const u16,
                self.num_transducers,
            );

            self.bram_write(
                BRAM_SELECT_CONTROLLER,
                ADDR_STM_FREQ_DIV0 + segment as u16,
                0xFFFF,
            );
            self.bram_write(
                BRAM_SELECT_CONTROLLER,
                ADDR_STM_REP0 + segment as u16,
                0xFFFF,
            );
            self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_STM_CYCLE0 + segment as u16, 0);
            self.bram_write(
                BRAM_SELECT_CONTROLLER,
                ADDR_STM_MODE0 + segment as u16,
                STM_MODE_GAIN,
            );

            self.stm_cycle[segment as usize] = 1;
            self.stm_rep[segment as usize] = 0xFFFF;
            self.stm_freq_div[segment as usize] = 0xFFFF;

            self.change_stm_wr_segment(segment as _);
            self.change_stm_wr_page(0);
            (0..self.num_transducers)
                .for_each(|i| self.bram_write(BRAM_SELECT_STM, i as _, data[i]));

            if (d.flag & GAIN_FLAG_UPDATE) == GAIN_FLAG_UPDATE {
                self.bram_write(
                    BRAM_SELECT_CONTROLLER,
                    ADDR_STM_REQ_RD_SEGMENT,
                    segment as _,
                );
                self.bram_write(
                    BRAM_SELECT_CONTROLLER,
                    ADDR_STM_TRANSITION_MODE,
                    TRANSITION_MODE_SYNC_IDX as _,
                );
                self.set_and_wait_update(CTL_FLAG_STM_SET);
            }

            NO_ERR
        }
    }

    #[must_use]
    pub(crate) unsafe fn change_gain_segment(&mut self, data: &[u8]) -> u8 {
        let d = Self::cast::<GainUpdate>(data);

        if self.stm_mode[d.segment as usize] != STM_MODE_GAIN
            || self.stm_cycle[d.segment as usize] != 1
        {
            return ERR_INVALID_SEGMENT_TRANSITION;
        }

        self.stm_segment = d.segment;

        self.bram_write(
            BRAM_SELECT_CONTROLLER,
            ADDR_STM_REQ_RD_SEGMENT,
            d.segment as _,
        );
        self.bram_write(
            BRAM_SELECT_CONTROLLER,
            ADDR_STM_TRANSITION_MODE,
            TRANSITION_MODE_SYNC_IDX as _,
        );
        self.set_and_wait_update(CTL_FLAG_STM_SET);

        NO_ERR
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mem_layout() {
        assert_eq!(4, std::mem::size_of::<Gain>());
        assert_eq!(0, std::mem::offset_of!(Gain, tag));
        assert_eq!(1, std::mem::offset_of!(Gain, segment));
        assert_eq!(2, std::mem::offset_of!(Gain, flag));

        assert_eq!(2, std::mem::size_of::<GainUpdate>());
        assert_eq!(0, std::mem::offset_of!(GainUpdate, tag));
        assert_eq!(1, std::mem::offset_of!(GainUpdate, segment));
    }
}
