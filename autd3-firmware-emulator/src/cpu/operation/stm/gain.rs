use crate::{cpu::params::*, CPUEmulator};

pub const GAIN_STM_BUF_PAGE_SIZE_WIDTH: u16 = 6;
pub const GAIN_STM_BUF_PAGE_SIZE: u16 = 1 << GAIN_STM_BUF_PAGE_SIZE_WIDTH;
pub const GAIN_STM_BUF_PAGE_SIZE_MASK: u16 = GAIN_STM_BUF_PAGE_SIZE - 1;

#[repr(C, align(2))]
#[derive(Clone, Copy)]
struct GainSTMHead {
    tag: u8,
    flag: u8,
    mode: u8,
    transition_mode: u8,
    freq_div: u16,
    rep: u16,
    transition_value: u64,
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
    transition_mode: u8,
    __: [u8; 5],
    transition_value: u64,
}

impl CPUEmulator {
    pub(crate) unsafe fn write_gain_stm(&mut self, data: &[u8]) -> u8 {
        let d = Self::cast::<GainSTM>(data);

        let segment = if (d.head.flag & GAIN_STM_FLAG_SEGMENT) != 0 {
            1
        } else {
            0
        };
        let send = (d.subseq.flag >> 6) + 1;

        let src_base = if (d.subseq.flag & GAIN_STM_FLAG_BEGIN) == GAIN_STM_FLAG_BEGIN {
            self.gain_stm_mode = d.head.mode;

            if Self::validate_transition_mode(
                self.stm_segment,
                segment,
                d.head.rep,
                d.head.transition_mode,
            ) {
                return ERR_INVALID_TRANSITION_MODE;
            }

            if self.validate_silencer_settings(
                d.head.freq_div,
                self.mod_freq_div[self.mod_segment as usize],
            ) {
                return ERR_INVALID_SILENCER_SETTING;
            }

            if d.head.transition_mode != TRANSITION_MODE_NONE {
                self.stm_segment = segment;
            }
            self.stm_cycle[segment as usize] = 0;
            self.stm_rep[segment as usize] = d.head.rep;
            self.stm_transition_mode = d.head.transition_mode;
            self.stm_transition_value = d.head.transition_value;
            self.stm_freq_div[segment as usize] = d.head.freq_div;

            match segment {
                0 => {
                    self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_STM_FREQ_DIV0, d.head.freq_div);
                    self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_STM_MODE0, STM_MODE_GAIN);
                    self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_STM_REP0, d.head.rep);
                }
                1 => {
                    self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_STM_FREQ_DIV1, d.head.freq_div);
                    self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_STM_MODE1, STM_MODE_GAIN);
                    self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_STM_REP1, d.head.rep);
                }
                _ => unreachable!(),
            }

            self.change_stm_wr_segment(segment as _);
            self.change_stm_wr_page(0);

            unsafe { data.as_ptr().add(std::mem::size_of::<GainSTMHead>()) as *const u16 }
        } else {
            unsafe { data.as_ptr().add(std::mem::size_of::<GainSTMSubseq>()) as *const u16 }
        };

        let mut src = src_base;
        let mut dst = (self.stm_cycle[segment as usize] & GAIN_STM_BUF_PAGE_SIZE_MASK) << 8;

        match self.gain_stm_mode {
            GAIN_STM_MODE_INTENSITY_PHASE_FULL => {
                self.stm_cycle[segment as usize] += 1;
                (0..self.num_transducers).for_each(|_| unsafe {
                    self.bram_write(BRAM_SELECT_STM, dst, src.read());
                    dst += 1;
                    src = src.add(1);
                });
            }
            GAIN_STM_MODE_PHASE_FULL => {
                (0..self.num_transducers).for_each(|_| unsafe {
                    self.bram_write(BRAM_SELECT_STM, dst, 0xFF00 | (src.read() & 0x00FF));
                    dst += 1;
                    src = src.add(1);
                });
                self.stm_cycle[segment as usize] += 1;

                if send > 1 {
                    let mut src = src_base;
                    let mut dst =
                        (self.stm_cycle[segment as usize] & GAIN_STM_BUF_PAGE_SIZE_MASK) << 8;
                    (0..self.num_transducers).for_each(|_| unsafe {
                        self.bram_write(
                            BRAM_SELECT_STM,
                            dst,
                            0xFF00 | ((src.read() >> 8) & 0x00FF),
                        );
                        dst += 1;
                        src = src.add(1);
                    });
                    self.stm_cycle[segment as usize] += 1;
                }
            }
            GAIN_STM_MODE_PHASE_HALF => {
                (0..self.num_transducers).for_each(|_| unsafe {
                    let phase = src.read() & 0x000F;
                    self.bram_write(BRAM_SELECT_STM, dst, 0xFF00 | (phase << 4) | phase);
                    dst += 1;
                    src = src.add(1);
                });
                self.stm_cycle[segment as usize] += 1;

                if send > 1 {
                    let mut src = src_base;
                    let mut dst =
                        (self.stm_cycle[segment as usize] & GAIN_STM_BUF_PAGE_SIZE_MASK) << 8;
                    (0..self.num_transducers).for_each(|_| unsafe {
                        let phase = (src.read() >> 4) & 0x000F;
                        self.bram_write(BRAM_SELECT_STM, dst, 0xFF00 | (phase << 4) | phase);
                        dst += 1;
                        src = src.add(1);
                    });
                    self.stm_cycle[segment as usize] += 1;
                }

                if send > 2 {
                    let mut src = src_base;
                    let mut dst =
                        (self.stm_cycle[segment as usize] & GAIN_STM_BUF_PAGE_SIZE_MASK) << 8;
                    (0..self.num_transducers).for_each(|_| unsafe {
                        let phase = (src.read() >> 8) & 0x000F;
                        self.bram_write(BRAM_SELECT_STM, dst, 0xFF00 | (phase << 4) | phase);
                        dst += 1;
                        src = src.add(1);
                    });
                    self.stm_cycle[segment as usize] += 1;
                }

                if send > 3 {
                    let mut src = src_base;
                    let mut dst =
                        (self.stm_cycle[segment as usize] & GAIN_STM_BUF_PAGE_SIZE_MASK) << 8;
                    (0..self.num_transducers).for_each(|_| unsafe {
                        let phase = (src.read() >> 12) & 0x000F;
                        self.bram_write(BRAM_SELECT_STM, dst, 0xFF00 | (phase << 4) | phase);
                        dst += 1;
                        src = src.add(1);
                    });
                    self.stm_cycle[segment as usize] += 1;
                }
            }
            _ => return ERR_INVALID_GAIN_STM_MODE,
        }

        if self.stm_cycle[segment as usize] & GAIN_STM_BUF_PAGE_SIZE_MASK == 0 {
            self.change_stm_wr_page(
                ((self.stm_cycle[segment as usize] & !GAIN_STM_BUF_PAGE_SIZE_MASK)
                    >> GAIN_STM_BUF_PAGE_SIZE_WIDTH) as _,
            );
        }

        if (d.subseq.flag & GAIN_STM_FLAG_END) == GAIN_STM_FLAG_END {
            self.stm_mode[segment as usize] = STM_MODE_GAIN;
            match segment {
                0 => {
                    self.bram_write(
                        BRAM_SELECT_CONTROLLER,
                        ADDR_STM_CYCLE0,
                        (self.stm_cycle[segment as usize].max(1) - 1) as _,
                    );
                }
                1 => {
                    self.bram_write(
                        BRAM_SELECT_CONTROLLER,
                        ADDR_STM_CYCLE1,
                        (self.stm_cycle[segment as usize].max(1) - 1) as _,
                    );
                }
                _ => unreachable!(),
            }

            if (d.subseq.flag & GAIN_STM_FLAG_UPDATE) == GAIN_STM_FLAG_UPDATE {
                return self.stm_segment_update(
                    segment,
                    self.stm_transition_mode,
                    self.stm_transition_value,
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

        if Self::validate_transition_mode(
            self.stm_segment,
            d.segment,
            self.stm_rep[d.segment as usize],
            d.transition_mode,
        ) {
            return ERR_INVALID_TRANSITION_MODE;
        }

        if self.validate_silencer_settings(
            self.stm_freq_div[d.segment as usize],
            self.mod_freq_div[self.mod_segment as usize],
        ) {
            return ERR_INVALID_SILENCER_SETTING;
        }

        self.stm_segment = d.segment;
        self.stm_segment_update(d.segment, d.transition_mode, d.transition_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gain_stm_memory_layout() {
        assert_eq!(16, std::mem::size_of::<GainSTMHead>());
        assert_eq!(0, std::mem::offset_of!(GainSTMHead, tag));
        assert_eq!(1, std::mem::offset_of!(GainSTMHead, flag));
        assert_eq!(2, std::mem::offset_of!(GainSTMHead, mode));
        assert_eq!(3, std::mem::offset_of!(GainSTMHead, transition_mode));
        assert_eq!(4, std::mem::offset_of!(GainSTMHead, freq_div));
        assert_eq!(6, std::mem::offset_of!(GainSTMHead, rep));
        assert_eq!(8, std::mem::offset_of!(GainSTMHead, transition_value));

        assert_eq!(2, std::mem::size_of::<GainSTMSubseq>());
        assert_eq!(0, std::mem::offset_of!(GainSTMSubseq, tag));
        assert_eq!(1, std::mem::offset_of!(GainSTMSubseq, flag));

        assert_eq!(0, std::mem::offset_of!(GainSTM, head));
        assert_eq!(0, std::mem::offset_of!(GainSTM, subseq));

        assert_eq!(16, std::mem::size_of::<GainSTMUpdate>());
        assert_eq!(0, std::mem::offset_of!(GainSTMUpdate, tag));
        assert_eq!(1, std::mem::offset_of!(GainSTMUpdate, segment));
        assert_eq!(2, std::mem::offset_of!(GainSTMUpdate, transition_mode));
        assert_eq!(8, std::mem::offset_of!(GainSTMUpdate, transition_value));
    }
}
