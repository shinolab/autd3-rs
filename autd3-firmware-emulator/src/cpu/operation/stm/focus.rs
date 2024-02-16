use crate::{cpu::params::*, CPUEmulator};

pub const FOCUS_STM_BUF_PAGE_SIZE_WIDTH: u32 = 12;
pub const FOCUS_STM_BUF_PAGE_SIZE: u32 = 1 << FOCUS_STM_BUF_PAGE_SIZE_WIDTH;
pub const FOCUS_STM_BUF_PAGE_SIZE_MASK: u32 = FOCUS_STM_BUF_PAGE_SIZE - 1;

#[repr(C)]
#[derive(Clone, Copy)]
struct FocusSTMHead {
    tag: u8,
    flag: u8,
    send_num: u8,
    segment: u8,
    freq_div: u32,
    sound_speed: u32,
    rep: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct FocusSTMSubseq {
    tag: u8,
    flag: u8,
    send_num: u8,
    pad: u8,
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
}

impl CPUEmulator {
    pub(crate) unsafe fn write_focus_stm(&mut self, data: &[u8]) -> u8 {
        let d = Self::cast::<FocusSTM>(data);

        let size = d.subseq.send_num as u32;

        let mut src = if (d.subseq.flag & FOCUS_STM_FLAG_BEGIN) == FOCUS_STM_FLAG_BEGIN {
            self.stm_cycle = 0;

            let freq_div = d.head.freq_div;
            let sound_speed = d.head.sound_speed;
            let rep = d.head.rep;
            let segment = d.head.segment;

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
                    self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_STM_MODE_0, STM_MODE_FOCUS);
                    self.bram_cpy(
                        BRAM_SELECT_CONTROLLER,
                        BRAM_ADDR_STM_SOUND_SPEED_0_0,
                        &sound_speed as *const _ as _,
                        std::mem::size_of::<u32>() >> 1,
                    );
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
                    self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_STM_MODE_1, STM_MODE_FOCUS);
                    self.bram_cpy(
                        BRAM_SELECT_CONTROLLER,
                        BRAM_ADDR_STM_SOUND_SPEED_1_0,
                        &sound_speed as *const _ as _,
                        std::mem::size_of::<u32>() >> 1,
                    );
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

            unsafe { data.as_ptr().add(std::mem::size_of::<FocusSTMHead>()) as *const u16 }
        } else {
            unsafe { data.as_ptr().add(std::mem::size_of::<FocusSTMSubseq>()) as *const u16 }
        };

        let page_capacity = (self.stm_cycle & !FOCUS_STM_BUF_PAGE_SIZE_MASK)
            + FOCUS_STM_BUF_PAGE_SIZE
            - self.stm_cycle;
        if size < page_capacity {
            let mut dst = ((self.stm_cycle & FOCUS_STM_BUF_PAGE_SIZE_MASK) << 2) as u16;
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
            self.stm_cycle += size;
        } else {
            let mut dst = ((self.stm_cycle & FOCUS_STM_BUF_PAGE_SIZE_MASK) << 2) as u16;
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
            self.stm_cycle += page_capacity;

            self.change_stm_wr_page(
                ((self.stm_cycle & !FOCUS_STM_BUF_PAGE_SIZE_MASK) >> FOCUS_STM_BUF_PAGE_SIZE_WIDTH)
                    as _,
            );

            let mut dst = ((self.stm_cycle & FOCUS_STM_BUF_PAGE_SIZE_MASK) << 2) as u16;
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
            self.stm_cycle += size - page_capacity;
        }

        if (d.subseq.flag & FOCUS_STM_FLAG_END) == FOCUS_STM_FLAG_END {
            match self.stm_segment {
                0 => {
                    self.bram_write(
                        BRAM_SELECT_CONTROLLER,
                        BRAM_ADDR_STM_CYCLE_0,
                        (self.stm_cycle.max(1) - 1) as _,
                    );
                }
                1 => {
                    self.bram_write(
                        BRAM_SELECT_CONTROLLER,
                        BRAM_ADDR_STM_CYCLE_1,
                        (self.stm_cycle.max(1) - 1) as _,
                    );
                }
                _ => unreachable!(),
            }

            if (d.subseq.flag & FOCUS_STM_FLAG_UPDATE) == FOCUS_STM_FLAG_UPDATE {
                self.bram_write(
                    BRAM_SELECT_CONTROLLER,
                    BRAM_ADDR_STM_REQ_RD_SEGMENT,
                    self.stm_segment as _,
                );
            }
        }

        NO_ERR
    }

    pub(crate) unsafe fn change_focus_stm_segment(&mut self, data: &[u8]) -> u8 {
        let d = Self::cast::<FocusSTMUpdate>(data);

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
    fn focus_stm_memory_layout() {
        assert_eq!(16, std::mem::size_of::<FocusSTMHead>());
        assert_eq!(0, memoffset::offset_of!(FocusSTMHead, tag));
        assert_eq!(1, memoffset::offset_of!(FocusSTMHead, flag));
        assert_eq!(2, memoffset::offset_of!(FocusSTMHead, send_num));
        assert_eq!(3, memoffset::offset_of!(FocusSTMHead, segment));
        assert_eq!(4, memoffset::offset_of!(FocusSTMHead, freq_div));
        assert_eq!(8, memoffset::offset_of!(FocusSTMHead, sound_speed));
        assert_eq!(12, memoffset::offset_of!(FocusSTMHead, rep));

        assert_eq!(4, std::mem::size_of::<FocusSTMSubseq>());
        assert_eq!(0, memoffset::offset_of!(FocusSTMSubseq, tag));
        assert_eq!(1, memoffset::offset_of!(FocusSTMSubseq, flag));
        assert_eq!(2, memoffset::offset_of!(FocusSTMSubseq, send_num));

        assert_eq!(0, memoffset::offset_of_union!(FocusSTM, head));
        assert_eq!(0, memoffset::offset_of_union!(FocusSTM, subseq));

        assert_eq!(2, std::mem::size_of::<FocusSTMUpdate>());
        assert_eq!(0, memoffset::offset_of!(FocusSTMUpdate, tag));
        assert_eq!(1, memoffset::offset_of!(FocusSTMUpdate, segment));
    }
}
