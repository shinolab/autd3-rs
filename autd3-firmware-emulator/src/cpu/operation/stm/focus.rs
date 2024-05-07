use crate::{cpu::params::*, CPUEmulator};

pub const FOCUS_STM_BUF_PAGE_SIZE_WIDTH: u32 = 12;
pub const FOCUS_STM_BUF_PAGE_SIZE: u32 = 1 << FOCUS_STM_BUF_PAGE_SIZE_WIDTH;
pub const FOCUS_STM_BUF_PAGE_SIZE_MASK: u32 = FOCUS_STM_BUF_PAGE_SIZE - 1;

#[repr(C, align(2))]
#[derive(Clone, Copy)]
struct FocusSTMHead {
    tag: u8,
    flag: u8,
    send_num: u8,
    transition_mode: u8,
    freq_div: u32,
    sound_speed: u32,
    rep: u32,
    transition_value: u64,
}

#[repr(C, align(2))]
#[derive(Clone, Copy)]
struct FocusSTMSubseq {
    tag: u8,
    flag: u8,
    send_num: u8,
    __pad: u8,
}

#[repr(C, align(2))]
union FocusSTM {
    head: FocusSTMHead,
    subseq: FocusSTMSubseq,
}

#[repr(C, align(2))]
#[derive(Clone, Copy)]
struct FocusSTMUpdate {
    tag: u8,
    segment: u8,
    transition_mode: u8,
    __pad: [u8; 5],
    transition_value: u64,
}

impl CPUEmulator {
    pub(crate) unsafe fn write_focus_stm(&mut self, data: &[u8]) -> u8 {
        let d = Self::cast::<FocusSTM>(data);

        let size = d.subseq.send_num as u32;

        let mut src = if (d.subseq.flag & FOCUS_STM_FLAG_BEGIN) == FOCUS_STM_FLAG_BEGIN {
            self.stm_segment = if (d.head.flag & GAIN_STM_FLAG_SEGMENT) != 0 {
                1
            } else {
                0
            };

            self.stm_cycle[self.stm_segment as usize] = 0;
            self.stm_transition_mode = d.head.transition_mode;
            self.stm_transition_value = d.head.transition_value;

            self.stm_freq_div[self.stm_segment as usize] = d.head.freq_div;
            if self.validate_silencer_settings() {
                return ERR_INVALID_SILENCER_SETTING;
            }

            match self.stm_segment {
                0 => {
                    self.bram_cpy(
                        BRAM_SELECT_CONTROLLER,
                        ADDR_STM_FREQ_DIV0_0,
                        &d.head.freq_div as *const _ as _,
                        std::mem::size_of::<u32>() >> 1,
                    );
                    self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_STM_MODE0, STM_MODE_FOCUS);
                    self.bram_cpy(
                        BRAM_SELECT_CONTROLLER,
                        ADDR_STM_SOUND_SPEED0_0,
                        &d.head.sound_speed as *const _ as _,
                        std::mem::size_of::<u32>() >> 1,
                    );
                    self.bram_cpy(
                        BRAM_SELECT_CONTROLLER,
                        ADDR_STM_REP0_0,
                        &d.head.rep as *const _ as _,
                        std::mem::size_of::<u32>() >> 1,
                    );
                }
                1 => {
                    self.bram_cpy(
                        BRAM_SELECT_CONTROLLER,
                        ADDR_STM_FREQ_DIV1_0,
                        &d.head.freq_div as *const _ as _,
                        std::mem::size_of::<u32>() >> 1,
                    );
                    self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_STM_MODE1, STM_MODE_FOCUS);
                    self.bram_cpy(
                        BRAM_SELECT_CONTROLLER,
                        ADDR_STM_SOUND_SPEED1_0,
                        &d.head.sound_speed as *const _ as _,
                        std::mem::size_of::<u32>() >> 1,
                    );
                    self.bram_cpy(
                        BRAM_SELECT_CONTROLLER,
                        ADDR_STM_REP1_0,
                        &d.head.rep as *const _ as _,
                        std::mem::size_of::<u32>() >> 1,
                    );
                }
                _ => unreachable!(),
            }

            self.change_stm_wr_segment(self.stm_segment as _);
            self.change_stm_wr_page(0);

            unsafe { data.as_ptr().add(std::mem::size_of::<FocusSTMHead>()) as *const u16 }
        } else {
            unsafe { data.as_ptr().add(std::mem::size_of::<FocusSTMSubseq>()) as *const u16 }
        };

        let page_capacity = (self.stm_cycle[self.stm_segment as usize]
            & !FOCUS_STM_BUF_PAGE_SIZE_MASK)
            + FOCUS_STM_BUF_PAGE_SIZE
            - self.stm_cycle[self.stm_segment as usize];
        if size <= page_capacity {
            let mut dst = ((self.stm_cycle[self.stm_segment as usize]
                & FOCUS_STM_BUF_PAGE_SIZE_MASK)
                << 2) as u16;
            (0..size as usize).for_each(|_| unsafe {
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
            self.stm_cycle[self.stm_segment as usize] += size;
        } else {
            let mut dst = ((self.stm_cycle[self.stm_segment as usize]
                & FOCUS_STM_BUF_PAGE_SIZE_MASK)
                << 2) as u16;
            (0..page_capacity as usize).for_each(|_| unsafe {
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
            self.stm_cycle[self.stm_segment as usize] += page_capacity;

            self.change_stm_wr_page(
                ((self.stm_cycle[self.stm_segment as usize] & !FOCUS_STM_BUF_PAGE_SIZE_MASK)
                    >> FOCUS_STM_BUF_PAGE_SIZE_WIDTH) as _,
            );

            let mut dst = ((self.stm_cycle[self.stm_segment as usize]
                & FOCUS_STM_BUF_PAGE_SIZE_MASK)
                << 2) as u16;
            let cnt = size - page_capacity;
            (0..cnt as usize).for_each(|_| unsafe {
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
            self.stm_cycle[self.stm_segment as usize] += size - page_capacity;
        }

        if (d.subseq.flag & FOCUS_STM_FLAG_END) == FOCUS_STM_FLAG_END {
            self.stm_mode[self.stm_segment as usize] = STM_MODE_FOCUS;
            match self.stm_segment {
                0 => {
                    self.bram_write(
                        BRAM_SELECT_CONTROLLER,
                        ADDR_STM_CYCLE0,
                        (self.stm_cycle[self.stm_segment as usize].max(1) - 1) as _,
                    );
                }
                1 => {
                    self.bram_write(
                        BRAM_SELECT_CONTROLLER,
                        ADDR_STM_CYCLE1,
                        (self.stm_cycle[self.stm_segment as usize].max(1) - 1) as _,
                    );
                }
                _ => unreachable!(),
            }

            if (d.subseq.flag & FOCUS_STM_FLAG_UPDATE) == FOCUS_STM_FLAG_UPDATE {
                return self.stm_segment_update(
                    self.stm_segment,
                    self.stm_transition_mode,
                    self.stm_transition_value,
                );
            }
        }

        NO_ERR
    }

    pub(crate) unsafe fn change_focus_stm_segment(&mut self, data: &[u8]) -> u8 {
        let d = Self::cast::<FocusSTMUpdate>(data);

        if self.stm_mode[d.segment as usize] != STM_MODE_FOCUS {
            return ERR_INVALID_SEGMENT_TRANSITION;
        }
        self.stm_segment = d.segment;
        if self.validate_silencer_settings() {
            return ERR_INVALID_SILENCER_SETTING;
        }

        self.stm_segment_update(d.segment, d.transition_mode, d.transition_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focus_stm_memory_layout() {
        assert_eq!(24, std::mem::size_of::<FocusSTMHead>());
        assert_eq!(0, std::mem::offset_of!(FocusSTMHead, tag));
        assert_eq!(1, std::mem::offset_of!(FocusSTMHead, flag));
        assert_eq!(2, std::mem::offset_of!(FocusSTMHead, send_num));
        assert_eq!(3, std::mem::offset_of!(FocusSTMHead, transition_mode));
        assert_eq!(4, std::mem::offset_of!(FocusSTMHead, freq_div));
        assert_eq!(8, std::mem::offset_of!(FocusSTMHead, sound_speed));
        assert_eq!(12, std::mem::offset_of!(FocusSTMHead, rep));
        assert_eq!(16, std::mem::offset_of!(FocusSTMHead, transition_value));

        assert_eq!(4, std::mem::size_of::<FocusSTMSubseq>());
        assert_eq!(0, std::mem::offset_of!(FocusSTMSubseq, tag));
        assert_eq!(1, std::mem::offset_of!(FocusSTMSubseq, flag));
        assert_eq!(2, std::mem::offset_of!(FocusSTMSubseq, send_num));

        assert_eq!(0, std::mem::offset_of!(FocusSTM, head));
        assert_eq!(0, std::mem::offset_of!(FocusSTM, subseq));

        assert_eq!(16, std::mem::size_of::<FocusSTMUpdate>());
        assert_eq!(0, std::mem::offset_of!(FocusSTMUpdate, tag));
        assert_eq!(1, std::mem::offset_of!(FocusSTMUpdate, segment));
        assert_eq!(2, std::mem::offset_of!(FocusSTMUpdate, transition_mode));
        assert_eq!(8, std::mem::offset_of!(FocusSTMUpdate, transition_value));
    }
}
