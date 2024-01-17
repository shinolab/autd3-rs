/*
 * File: focus.rs
 * Project: stm
 * Created Date: 17/01/2024
 * Author: Shun Suzuki
 * -----
 * Last Modified: 17/01/2024
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2024 Shun Suzuki. All rights reserved.
 *
 */

use crate::{
    cpu::params::{
        BRAM_ADDR_SOUND_SPEED_0, BRAM_ADDR_STM_ADDR_OFFSET, BRAM_ADDR_STM_CYCLE,
        BRAM_ADDR_STM_FINISH_IDX, BRAM_ADDR_STM_FREQ_DIV_0, BRAM_ADDR_STM_START_IDX,
        BRAM_SELECT_CONTROLLER, BRAM_SELECT_STM, CTL_FLAG_OP_MODE, CTL_FLAG_USE_STM_FINISH_IDX,
        CTL_FLAG_USE_STM_START_IDX, CTL_REG_STM_GAIN_MODE, ERR_FREQ_DIV_TOO_SMALL, ERR_NONE,
    },
    CPUEmulator,
};

const FOCUS_STM_FLAG_BEGIN: u8 = 1 << 0;
const FOCUS_STM_FLAG_END: u8 = 1 << 1;
const FOCUS_STM_FLAG_USE_START_IDX: u8 = 1 << 2;
const FOCUS_STM_FLAG_USE_FINISH_IDX: u8 = 1 << 3;

pub const FOCUS_STM_BUF_PAGE_SIZE_WIDTH: u32 = 11;
pub const FOCUS_STM_BUF_PAGE_SIZE: u32 = 1 << FOCUS_STM_BUF_PAGE_SIZE_WIDTH;
pub const FOCUS_STM_BUF_PAGE_SIZE_MASK: u32 = FOCUS_STM_BUF_PAGE_SIZE - 1;

#[repr(C)]
#[derive(Clone, Copy)]
struct FocusSTMHead {
    tag: u8,
    flag: u8,
    send_num: u16,
    freq_div: u32,
    sound_speed: u32,
    start_idx: u16,
    finish_idx: u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct FocusSTMSubseq {
    tag: u8,
    flag: u8,
    send_num: u16,
}

#[repr(C, align(2))]
union FocusSTM {
    head: FocusSTMHead,
    subseq: FocusSTMSubseq,
}

impl CPUEmulator {
    pub(crate) unsafe fn write_focus_stm(&mut self, data: &[u8]) -> u8 {
        let d = Self::cast::<FocusSTM>(data);

        let mut src = if (d.subseq.flag & FOCUS_STM_FLAG_BEGIN) == FOCUS_STM_FLAG_BEGIN {
            self.stm_cycle = 0;
            self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_STM_ADDR_OFFSET, 0);
            let freq_div = d.head.freq_div;
            if self.silencer_strict_mode
                && ((freq_div < self.min_freq_div_intensity)
                    || (freq_div < self.min_freq_div_phase))
            {
                return ERR_FREQ_DIV_TOO_SMALL;
            }
            self.stm_freq_div = freq_div;

            let sound_speed = d.head.sound_speed;
            let start_idx = d.head.start_idx;
            let finish_idx = d.head.finish_idx;

            self.bram_cpy(
                BRAM_SELECT_CONTROLLER,
                BRAM_ADDR_STM_FREQ_DIV_0,
                &freq_div as *const _ as _,
                std::mem::size_of::<u32>() >> 1,
            );
            self.bram_cpy(
                BRAM_SELECT_CONTROLLER,
                BRAM_ADDR_SOUND_SPEED_0,
                &sound_speed as *const _ as _,
                std::mem::size_of::<u32>() >> 1,
            );

            self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_STM_START_IDX, start_idx);
            self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_STM_FINISH_IDX, finish_idx);

            if (d.head.flag & FOCUS_STM_FLAG_USE_START_IDX) == FOCUS_STM_FLAG_USE_START_IDX {
                self.fpga_flags_internal |= CTL_FLAG_USE_STM_START_IDX;
            } else {
                self.fpga_flags_internal &= !CTL_FLAG_USE_STM_START_IDX;
            }
            if (d.head.flag & FOCUS_STM_FLAG_USE_FINISH_IDX) == FOCUS_STM_FLAG_USE_FINISH_IDX {
                self.fpga_flags_internal |= CTL_FLAG_USE_STM_FINISH_IDX;
            } else {
                self.fpga_flags_internal &= !CTL_FLAG_USE_STM_FINISH_IDX;
            }

            unsafe { data.as_ptr().add(std::mem::size_of::<FocusSTMHead>()) as *const u16 }
        } else {
            unsafe { data.as_ptr().add(std::mem::size_of::<FocusSTMSubseq>()) as *const u16 }
        };

        let size = d.subseq.send_num as u32;
        let page_capacity = (self.stm_cycle & !FOCUS_STM_BUF_PAGE_SIZE_MASK)
            + FOCUS_STM_BUF_PAGE_SIZE
            - self.stm_cycle;
        if size < page_capacity {
            let mut dst = ((self.stm_cycle & FOCUS_STM_BUF_PAGE_SIZE_MASK) << 3) as u16;
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
                dst += 4;
            });
            self.stm_cycle += size;
        } else {
            let mut dst = ((self.stm_cycle & FOCUS_STM_BUF_PAGE_SIZE_MASK) << 3) as u16;
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
                dst += 4;
            });
            self.stm_cycle += page_capacity;

            self.bram_write(
                BRAM_SELECT_CONTROLLER,
                BRAM_ADDR_STM_ADDR_OFFSET,
                ((self.stm_cycle & !FOCUS_STM_BUF_PAGE_SIZE_MASK) >> FOCUS_STM_BUF_PAGE_SIZE_WIDTH)
                    as _,
            );

            let mut dst = ((self.stm_cycle & FOCUS_STM_BUF_PAGE_SIZE_MASK) << 3) as u16;
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
                dst += 4;
            });
            self.stm_cycle += size - page_capacity;
        }

        if (d.subseq.flag & FOCUS_STM_FLAG_END) == FOCUS_STM_FLAG_END {
            self.fpga_flags_internal |= CTL_FLAG_OP_MODE;
            self.fpga_flags_internal &= !CTL_REG_STM_GAIN_MODE;
            self.bram_write(
                BRAM_SELECT_CONTROLLER,
                BRAM_ADDR_STM_CYCLE,
                (self.stm_cycle.max(1) - 1) as _,
            );
        }

        ERR_NONE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focus_stm_derive() {
        assert_eq!(std::mem::size_of::<FocusSTMHead>(), 16);
        assert_eq!(memoffset::offset_of!(FocusSTMHead, tag), 0);
        assert_eq!(memoffset::offset_of!(FocusSTMHead, flag), 1);
        assert_eq!(memoffset::offset_of!(FocusSTMHead, send_num), 2);
        assert_eq!(memoffset::offset_of!(FocusSTMHead, freq_div), 4);
        assert_eq!(memoffset::offset_of!(FocusSTMHead, sound_speed), 8);
        assert_eq!(memoffset::offset_of!(FocusSTMHead, start_idx), 12);
        assert_eq!(memoffset::offset_of!(FocusSTMHead, finish_idx), 14);

        assert_eq!(std::mem::size_of::<FocusSTMSubseq>(), 4);
        assert_eq!(memoffset::offset_of!(FocusSTMSubseq, tag), 0);
        assert_eq!(memoffset::offset_of!(FocusSTMSubseq, flag), 1);
        assert_eq!(memoffset::offset_of!(FocusSTMSubseq, send_num), 2);

        assert_eq!(memoffset::offset_of_union!(FocusSTM, head), 0);
        assert_eq!(memoffset::offset_of_union!(FocusSTM, subseq), 0);

        let head = FocusSTMHead {
            tag: 0,
            flag: 0,
            send_num: 0,
            freq_div: 0,
            sound_speed: 0,
            start_idx: 0,
            finish_idx: 0,
        };
        let _ = head.clone();

        let subseq = FocusSTMSubseq {
            tag: 0,
            flag: 0,
            send_num: 0,
        };
        let _ = subseq.clone();
    }
}
