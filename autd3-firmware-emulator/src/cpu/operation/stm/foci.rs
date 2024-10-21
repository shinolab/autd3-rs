use crate::{cpu::params::*, CPUEmulator};

pub const FOCI_STM_BUF_PAGE_SIZE_WIDTH: u16 = 9;
pub const FOCI_STM_BUF_PAGE_SIZE: u16 = 1 << FOCI_STM_BUF_PAGE_SIZE_WIDTH;
pub const FOCI_STM_BUF_PAGE_SIZE_MASK: u16 = FOCI_STM_BUF_PAGE_SIZE - 1;

#[repr(C, align(2))]
#[derive(Clone, Copy)]
struct FociSTMHead {
    tag: u8,
    flag: u8,
    send_num: u8,
    segment: u8,
    transition_mode: u8,
    num_foci: u8,
    sound_speed: u16,
    freq_div: u16,
    rep: u16,
    __: [u8; 4],
    transition_value: u64,
}

#[repr(C, align(2))]
#[derive(Clone, Copy)]
struct FociSTMSubseq {
    tag: u8,
    flag: u8,
    send_num: u8,
    segment: u8,
}

#[repr(C, align(2))]
union FociSTM {
    head: FociSTMHead,
    subseq: FociSTMSubseq,
}

#[repr(C, align(2))]
#[derive(Clone, Copy)]
struct FociSTMUpdate {
    tag: u8,
    segment: u8,
    transition_mode: u8,
    __: [u8; 5],
    transition_value: u64,
}

impl CPUEmulator {
    pub(crate) unsafe fn write_foci_stm(&mut self, data: &[u8]) -> u8 {
        let d = Self::cast::<FociSTM>(data);

        let segment = d.subseq.segment;
        let size = d.subseq.send_num as u16;

        let mut src = if (d.subseq.flag & FOCI_STM_FLAG_BEGIN) == FOCI_STM_FLAG_BEGIN {
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
            self.num_foci = d.head.num_foci;

            match segment {
                0 => {
                    self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_STM_FREQ_DIV0, d.head.freq_div);
                    self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_STM_MODE0, STM_MODE_FOCUS);
                    self.bram_write(
                        BRAM_SELECT_CONTROLLER,
                        ADDR_STM_SOUND_SPEED0,
                        d.head.sound_speed,
                    );
                    self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_STM_REP0, d.head.rep);
                    self.bram_write(
                        BRAM_SELECT_CONTROLLER,
                        ADDR_STM_NUM_FOCI0,
                        d.head.num_foci as _,
                    );
                }
                1 => {
                    self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_STM_FREQ_DIV1, d.head.freq_div);
                    self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_STM_MODE1, STM_MODE_FOCUS);
                    self.bram_write(
                        BRAM_SELECT_CONTROLLER,
                        ADDR_STM_SOUND_SPEED1,
                        d.head.sound_speed,
                    );
                    self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_STM_REP1, d.head.rep);
                    self.bram_write(
                        BRAM_SELECT_CONTROLLER,
                        ADDR_STM_NUM_FOCI1,
                        d.head.num_foci as _,
                    );
                }
                _ => unreachable!(),
            }

            self.change_stm_wr_segment(segment as _);
            self.change_stm_wr_page(0);

            unsafe { data.as_ptr().add(std::mem::size_of::<FociSTMHead>()) as *const u16 }
        } else {
            unsafe { data.as_ptr().add(std::mem::size_of::<FociSTMSubseq>()) as *const u16 }
        };

        let page_capacity = (self.stm_cycle[segment as usize] & !FOCI_STM_BUF_PAGE_SIZE_MASK)
            + FOCI_STM_BUF_PAGE_SIZE
            - self.stm_cycle[segment as usize];
        if size < page_capacity {
            let mut dst = (self.stm_cycle[segment as usize] & FOCI_STM_BUF_PAGE_SIZE_MASK) << 5;
            (0..size as usize).for_each(|_| {
                (0..self.num_foci).for_each(|_| unsafe {
                    self.bram_write(BRAM_SELECT_STM, dst, src.read());
                    dst += 1;
                    src = src.add(1);
                    self.bram_write(BRAM_SELECT_STM, dst, src.read());
                    dst += 1;
                    src = src.add(1);
                    self.bram_write(BRAM_SELECT_STM, dst, src.read());
                    dst += 1;
                    src = src.add(1);
                    self.bram_write(BRAM_SELECT_STM, dst, src.read());
                    dst += 1;
                    src = src.add(1);
                });
                dst += 4 * (8 - self.num_foci as u16);
            });
            self.stm_cycle[segment as usize] += size;
        } else {
            let mut dst = (self.stm_cycle[segment as usize] & FOCI_STM_BUF_PAGE_SIZE_MASK) << 5;
            (0..page_capacity as usize).for_each(|_| {
                (0..self.num_foci).for_each(|_| unsafe {
                    self.bram_write(BRAM_SELECT_STM, dst, src.read());
                    dst += 1;
                    src = src.add(1);
                    self.bram_write(BRAM_SELECT_STM, dst, src.read());
                    dst += 1;
                    src = src.add(1);
                    self.bram_write(BRAM_SELECT_STM, dst, src.read());
                    dst += 1;
                    src = src.add(1);
                    self.bram_write(BRAM_SELECT_STM, dst, src.read());
                    dst += 1;
                    src = src.add(1);
                });
                dst += 4 * (8 - self.num_foci as u16);
            });
            self.stm_cycle[segment as usize] += page_capacity;

            self.change_stm_wr_page(
                ((self.stm_cycle[segment as usize] & !FOCI_STM_BUF_PAGE_SIZE_MASK)
                    >> FOCI_STM_BUF_PAGE_SIZE_WIDTH) as _,
            );

            let mut dst = (self.stm_cycle[segment as usize] & FOCI_STM_BUF_PAGE_SIZE_MASK) << 5;
            let cnt = size - page_capacity;
            (0..cnt as usize).for_each(|_| {
                (0..self.num_foci).for_each(|_| unsafe {
                    self.bram_write(BRAM_SELECT_STM, dst, src.read());
                    dst += 1;
                    src = src.add(1);
                    self.bram_write(BRAM_SELECT_STM, dst, src.read());
                    dst += 1;
                    src = src.add(1);
                    self.bram_write(BRAM_SELECT_STM, dst, src.read());
                    dst += 1;
                    src = src.add(1);
                    self.bram_write(BRAM_SELECT_STM, dst, src.read());
                    dst += 1;
                    src = src.add(1);
                });
                dst += 4 * (8 - self.num_foci as u16);
            });
            self.stm_cycle[segment as usize] += size - page_capacity;
        }

        if (d.subseq.flag & FOCI_STM_FLAG_END) == FOCI_STM_FLAG_END {
            self.stm_mode[segment as usize] = STM_MODE_FOCUS;
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

            if (d.subseq.flag & FOCI_STM_FLAG_UPDATE) == FOCI_STM_FLAG_UPDATE {
                return self.stm_segment_update(
                    segment,
                    self.stm_transition_mode,
                    self.stm_transition_value,
                );
            }
        }

        NO_ERR
    }

    pub(crate) unsafe fn change_foci_stm_segment(&mut self, data: &[u8]) -> u8 {
        let d = Self::cast::<FociSTMUpdate>(data);

        if self.stm_mode[d.segment as usize] != STM_MODE_FOCUS {
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
    #[cfg_attr(miri, ignore)]
    fn foci_stm_memory_layout() {
        assert_eq!(24, std::mem::size_of::<FociSTMHead>());
        assert_eq!(0, std::mem::offset_of!(FociSTMHead, tag));
        assert_eq!(1, std::mem::offset_of!(FociSTMHead, flag));
        assert_eq!(2, std::mem::offset_of!(FociSTMHead, send_num));
        assert_eq!(3, std::mem::offset_of!(FociSTMHead, segment));
        assert_eq!(4, std::mem::offset_of!(FociSTMHead, transition_mode));
        assert_eq!(5, std::mem::offset_of!(FociSTMHead, num_foci));
        assert_eq!(6, std::mem::offset_of!(FociSTMHead, sound_speed));
        assert_eq!(8, std::mem::offset_of!(FociSTMHead, freq_div));
        assert_eq!(10, std::mem::offset_of!(FociSTMHead, rep));
        assert_eq!(16, std::mem::offset_of!(FociSTMHead, transition_value));

        assert_eq!(4, std::mem::size_of::<FociSTMSubseq>());
        assert_eq!(0, std::mem::offset_of!(FociSTMSubseq, tag));
        assert_eq!(1, std::mem::offset_of!(FociSTMSubseq, flag));
        assert_eq!(2, std::mem::offset_of!(FociSTMSubseq, send_num));
        assert_eq!(3, std::mem::offset_of!(FociSTMSubseq, segment));

        assert_eq!(0, std::mem::offset_of!(FociSTM, head));
        assert_eq!(0, std::mem::offset_of!(FociSTM, subseq));

        assert_eq!(16, std::mem::size_of::<FociSTMUpdate>());
        assert_eq!(0, std::mem::offset_of!(FociSTMUpdate, tag));
        assert_eq!(1, std::mem::offset_of!(FociSTMUpdate, segment));
        assert_eq!(2, std::mem::offset_of!(FociSTMUpdate, transition_mode));
        assert_eq!(8, std::mem::offset_of!(FociSTMUpdate, transition_value));
    }
}
