use crate::{cpu::params::*, CPUEmulator};

pub const GAIN_STM_BUF_PAGE_SIZE_WIDTH: u32 = 6;
pub const GAIN_STM_BUF_PAGE_SIZE: u32 = 1 << GAIN_STM_BUF_PAGE_SIZE_WIDTH;
pub const GAIN_STM_BUF_PAGE_SIZE_MASK: u32 = GAIN_STM_BUF_PAGE_SIZE - 1;

#[repr(C)]
#[derive(Clone, Copy)]
struct GainSTMHead {
    tag: u8,
    flag: u8,
    mode: u8,
    segment: u8,
    freq_div: u32,
    rep: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct GainSTMSubseq {
    tag: u8,
    flag: u8,
}

#[repr(C, align(2))]
union GainSTM {
    head: GainSTMHead,
    subseq: GainSTMSubseq,
}

#[repr(C, align(2))]
#[derive(Clone, Copy)]
struct GainSTMUpdate {
    tag: u8,
    segment: u8,
}

impl CPUEmulator {
    pub(crate) unsafe fn write_gain_stm(&mut self, data: &[u8]) -> u8 {
        let d = Self::cast::<GainSTM>(data);

        let send = (d.subseq.flag >> 6) + 1;

        let src_base = if (d.subseq.flag & GAIN_STM_FLAG_BEGIN) == GAIN_STM_FLAG_BEGIN {
            self.gain_stm_mode = d.head.mode;

            let rep = d.head.rep;
            let segment = d.head.segment;
            let freq_div = d.head.freq_div;

            self.stm_cycle[segment as usize] = 0;

            if self.silencer_strict_mode
                && ((freq_div < self.min_freq_div_intensity)
                    || (freq_div < self.min_freq_div_phase))
            {
                return ERR_FREQ_DIV_TOO_SMALL;
            }
            self.stm_freq_div[segment as usize] = freq_div;

            match segment {
                0 => {
                    self.bram_cpy(
                        BRAM_SELECT_CONTROLLER,
                        BRAM_ADDR_STM_FREQ_DIV_0_0,
                        &freq_div as *const _ as _,
                        std::mem::size_of::<u32>() >> 1,
                    );
                    self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_STM_MODE_0, STM_MODE_GAIN);
                    self.bram_cpy(
                        BRAM_SELECT_CONTROLLER,
                        BRAM_ADDR_STM_REP_0_0,
                        &rep as *const _ as _,
                        std::mem::size_of::<u32>() >> 1,
                    );
                }
                1 => {
                    self.bram_cpy(
                        BRAM_SELECT_CONTROLLER,
                        BRAM_ADDR_STM_FREQ_DIV_1_0,
                        &freq_div as *const _ as _,
                        std::mem::size_of::<u32>() >> 1,
                    );
                    self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_STM_MODE_1, STM_MODE_GAIN);
                    self.bram_cpy(
                        BRAM_SELECT_CONTROLLER,
                        BRAM_ADDR_STM_REP_1_0,
                        &rep as *const _ as _,
                        std::mem::size_of::<u32>() >> 1,
                    );
                }
                _ => return ERR_INVALID_SEGMENT,
            }
            self.stm_segment = segment;

            self.change_stm_wr_segment(segment as _);
            self.change_stm_wr_page(0);

            unsafe { data.as_ptr().add(12) as *const u16 }
        } else {
            unsafe { data.as_ptr().add(2) as *const u16 }
        };

        let mut src = src_base;
        let mut dst =
            ((self.stm_cycle[self.stm_segment as usize] & GAIN_STM_BUF_PAGE_SIZE_MASK) << 8) as u16;

        if self.gain_stm_mode == GAIN_STM_MODE_INTENSITY_PHASE_FULL {
            self.stm_cycle[self.stm_segment as usize] += 1;
            (0..self.num_transducers).for_each(|_| unsafe {
                self.bram_write(BRAM_SELECT_STM, dst, src.read());
                dst += 1;
                src = src.add(1);
            });
        } else if self.gain_stm_mode == GAIN_STM_MODE_PHASE_FULL {
            (0..self.num_transducers).for_each(|_| unsafe {
                self.bram_write(BRAM_SELECT_STM, dst, 0xFF00 | (src.read() & 0x00FF));
                dst += 1;
                src = src.add(1);
            });
            self.stm_cycle[self.stm_segment as usize] += 1;

            if send > 1 {
                let mut src = src_base;
                let mut dst = ((self.stm_cycle[self.stm_segment as usize]
                    & GAIN_STM_BUF_PAGE_SIZE_MASK)
                    << 8) as u16;
                (0..self.num_transducers).for_each(|_| unsafe {
                    self.bram_write(BRAM_SELECT_STM, dst, 0xFF00 | ((src.read() >> 8) & 0x00FF));
                    dst += 1;
                    src = src.add(1);
                });
                self.stm_cycle[self.stm_segment as usize] += 1;
            }
        } else if self.gain_stm_mode == GAIN_STM_MODE_PHASE_HALF {
            (0..self.num_transducers).for_each(|_| unsafe {
                let phase = src.read() & 0x000F;
                self.bram_write(BRAM_SELECT_STM, dst, 0xFF00 | (phase << 4) | phase);
                dst += 1;
                src = src.add(1);
            });
            self.stm_cycle[self.stm_segment as usize] += 1;

            if send > 1 {
                let mut src = src_base;
                let mut dst = ((self.stm_cycle[self.stm_segment as usize]
                    & GAIN_STM_BUF_PAGE_SIZE_MASK)
                    << 8) as u16;
                (0..self.num_transducers).for_each(|_| unsafe {
                    let phase = (src.read() >> 4) & 0x000F;
                    self.bram_write(BRAM_SELECT_STM, dst, 0xFF00 | (phase << 4) | phase);
                    dst += 1;
                    src = src.add(1);
                });
                self.stm_cycle[self.stm_segment as usize] += 1;
            }

            if send > 2 {
                let mut src = src_base;
                let mut dst = ((self.stm_cycle[self.stm_segment as usize]
                    & GAIN_STM_BUF_PAGE_SIZE_MASK)
                    << 8) as u16;
                (0..self.num_transducers).for_each(|_| unsafe {
                    let phase = (src.read() >> 8) & 0x000F;
                    self.bram_write(BRAM_SELECT_STM, dst, 0xFF00 | (phase << 4) | phase);
                    dst += 1;
                    src = src.add(1);
                });
                self.stm_cycle[self.stm_segment as usize] += 1;
            }

            if send > 3 {
                let mut src = src_base;
                let mut dst = ((self.stm_cycle[self.stm_segment as usize]
                    & GAIN_STM_BUF_PAGE_SIZE_MASK)
                    << 8) as u16;
                (0..self.num_transducers).for_each(|_| unsafe {
                    let phase = (src.read() >> 12) & 0x000F;
                    self.bram_write(BRAM_SELECT_STM, dst, 0xFF00 | (phase << 4) | phase);
                    dst += 1;
                    src = src.add(1);
                });
                self.stm_cycle[self.stm_segment as usize] += 1;
            }
        }

        if self.stm_cycle[self.stm_segment as usize] & GAIN_STM_BUF_PAGE_SIZE_MASK == 0 {
            self.change_stm_wr_page(
                ((self.stm_cycle[self.stm_segment as usize] & !GAIN_STM_BUF_PAGE_SIZE_MASK)
                    >> GAIN_STM_BUF_PAGE_SIZE_WIDTH) as _,
            );
        }

        if (d.subseq.flag & GAIN_STM_FLAG_END) == GAIN_STM_FLAG_END {
            self.stm_mode[self.stm_segment as usize] = STM_MODE_GAIN;
            match self.stm_segment {
                0 => {
                    self.bram_write(
                        BRAM_SELECT_CONTROLLER,
                        BRAM_ADDR_STM_CYCLE_0,
                        (self.stm_cycle[self.stm_segment as usize].max(1) - 1) as _,
                    );
                }
                1 => {
                    self.bram_write(
                        BRAM_SELECT_CONTROLLER,
                        BRAM_ADDR_STM_CYCLE_1,
                        (self.stm_cycle[self.stm_segment as usize].max(1) - 1) as _,
                    );
                }
                _ => unreachable!(),
            }

            if (d.subseq.flag & GAIN_STM_FLAG_UPDATE) == GAIN_STM_FLAG_UPDATE {
                self.bram_write(
                    BRAM_SELECT_CONTROLLER,
                    BRAM_ADDR_STM_REQ_RD_SEGMENT,
                    self.stm_segment as _,
                );
            }
        }

        NO_ERR
    }

    pub(crate) unsafe fn change_gain_stm_segment(&mut self, data: &[u8]) -> u8 {
        let d = Self::cast::<GainSTMUpdate>(data);

        if self.stm_mode[d.segment as usize] != STM_MODE_GAIN
            || self.stm_cycle[d.segment as usize] == 1
        {
            return ERR_INVALID_SEGMENT_TRANSITION;
        }

        self.bram_write(
            BRAM_SELECT_CONTROLLER,
            BRAM_ADDR_STM_REQ_RD_SEGMENT,
            d.segment as _,
        );

        NO_ERR
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gain_stm_memory_layout() {
        assert_eq!(12, std::mem::size_of::<GainSTMHead>());
        assert_eq!(0, std::mem::offset_of!(GainSTMHead, tag));
        assert_eq!(1, std::mem::offset_of!(GainSTMHead, flag));
        assert_eq!(2, std::mem::offset_of!(GainSTMHead, mode));
        assert_eq!(3, std::mem::offset_of!(GainSTMHead, segment));
        assert_eq!(4, std::mem::offset_of!(GainSTMHead, freq_div));
        assert_eq!(8, std::mem::offset_of!(GainSTMHead, rep));

        assert_eq!(2, std::mem::size_of::<GainSTMSubseq>());
        assert_eq!(0, std::mem::offset_of!(GainSTMSubseq, tag));
        assert_eq!(1, std::mem::offset_of!(GainSTMSubseq, flag));

        assert_eq!(0, std::mem::offset_of!(GainSTM, head));
        assert_eq!(0, std::mem::offset_of!(GainSTM, subseq));

        assert_eq!(2, std::mem::size_of::<GainSTMUpdate>());
        assert_eq!(0, std::mem::offset_of!(GainSTMUpdate, tag));
        assert_eq!(1, std::mem::offset_of!(GainSTMUpdate, segment));
    }
}
