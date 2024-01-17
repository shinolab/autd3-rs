/*
 * File: modulation.rs
 * Project: operation
 * Created Date: 30/12/2023
 * Author: Shun Suzuki
 * -----
 * Last Modified: 17/01/2024
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2023 Shun Suzuki. All rights reserved.
 *
 */

use crate::{
    cpu::params::{
        BRAM_ADDR_MOD_ADDR_OFFSET, BRAM_ADDR_MOD_CYCLE, BRAM_ADDR_MOD_FREQ_DIV_0,
        BRAM_SELECT_CONTROLLER, BRAM_SELECT_MOD, ERR_FREQ_DIV_TOO_SMALL, ERR_NONE,
    },
    CPUEmulator,
};

const MOD_BUF_PAGE_SIZE_WIDTH: u32 = 15;
const MOD_BUF_PAGE_SIZE: u32 = 1 << MOD_BUF_PAGE_SIZE_WIDTH;
const MOD_BUF_PAGE_SIZE_MASK: u32 = MOD_BUF_PAGE_SIZE - 1;

const MODULATION_FLAG_BEGIN: u8 = 1 << 0;
const MODULATION_FLAG_END: u8 = 1 << 1;

#[repr(C, align(2))]
#[derive(Clone, Copy)]
struct ModulationHead {
    tag: u8,
    flag: u8,
    size: u16,
    freq_div: u32,
}

#[repr(C, align(2))]
#[derive(Clone, Copy)]
struct ModulationSubseq {
    tag: u8,
    flag: u8,
    size: u16,
}

#[repr(C, align(2))]
union Modulation {
    head: ModulationHead,
    subseq: ModulationSubseq,
}

impl CPUEmulator {
    pub(crate) unsafe fn write_mod(&mut self, data: &[u8]) -> u8 {
        let d = Self::cast::<Modulation>(data);

        let data = if (d.subseq.flag & MODULATION_FLAG_BEGIN) == MODULATION_FLAG_BEGIN {
            self.mod_cycle = 0;
            self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_MOD_ADDR_OFFSET, 0);
            let freq_div = d.head.freq_div;
            if self.silencer_strict_mode & (freq_div < self.min_freq_div_intensity) {
                return ERR_FREQ_DIV_TOO_SMALL;
            }
            self.mod_freq_div = freq_div;

            self.bram_cpy(
                BRAM_SELECT_CONTROLLER,
                BRAM_ADDR_MOD_FREQ_DIV_0,
                &freq_div as *const _ as _,
                std::mem::size_of::<u32>() >> 1,
            );
            data[std::mem::size_of::<ModulationHead>()..].as_ptr() as *const u16
        } else {
            data[std::mem::size_of::<ModulationSubseq>()..].as_ptr() as *const u16
        };

        let page_capacity =
            (self.mod_cycle & !MOD_BUF_PAGE_SIZE_MASK) + MOD_BUF_PAGE_SIZE - self.mod_cycle;

        let write = d.subseq.size as u32;
        if write < page_capacity {
            self.bram_cpy(
                BRAM_SELECT_MOD,
                ((self.mod_cycle & MOD_BUF_PAGE_SIZE_MASK) >> 1) as u16,
                data,
                ((write + 1) >> 1) as usize,
            );
            self.mod_cycle += write;
        } else {
            self.bram_cpy(
                BRAM_SELECT_MOD,
                ((self.mod_cycle & MOD_BUF_PAGE_SIZE_MASK) >> 1) as u16,
                data,
                (page_capacity >> 1) as usize,
            );
            self.mod_cycle += page_capacity;
            let data = unsafe { data.add((page_capacity >> 1) as _) };
            self.bram_write(
                BRAM_SELECT_CONTROLLER,
                BRAM_ADDR_MOD_ADDR_OFFSET,
                ((self.mod_cycle & !MOD_BUF_PAGE_SIZE_MASK) >> MOD_BUF_PAGE_SIZE_WIDTH) as u16,
            );
            self.bram_cpy(
                BRAM_SELECT_MOD,
                ((self.mod_cycle & MOD_BUF_PAGE_SIZE_MASK) >> 1) as _,
                data,
                ((write - page_capacity + 1) >> 1) as _,
            );
            self.mod_cycle += write - page_capacity;
        }

        if (d.subseq.flag & MODULATION_FLAG_END) == MODULATION_FLAG_END {
            self.bram_write(
                BRAM_SELECT_CONTROLLER,
                BRAM_ADDR_MOD_CYCLE,
                (self.mod_cycle.max(1) - 1) as _,
            );
        }

        ERR_NONE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modulation_derive() {
        assert_eq!(std::mem::size_of::<ModulationHead>(), 8);
        assert_eq!(memoffset::offset_of!(ModulationHead, tag), 0);
        assert_eq!(memoffset::offset_of!(ModulationHead, flag), 1);
        assert_eq!(memoffset::offset_of!(ModulationHead, size), 2);
        assert_eq!(memoffset::offset_of!(ModulationHead, freq_div), 4);

        assert_eq!(std::mem::size_of::<ModulationSubseq>(), 4);
        assert_eq!(memoffset::offset_of!(ModulationSubseq, tag), 0);
        assert_eq!(memoffset::offset_of!(ModulationSubseq, flag), 1);
        assert_eq!(memoffset::offset_of!(ModulationSubseq, size), 2);

        assert_eq!(std::mem::size_of::<Modulation>(), 8);
        assert_eq!(memoffset::offset_of_union!(Modulation, head), 0);
        assert_eq!(memoffset::offset_of_union!(Modulation, subseq), 0);

        let head = ModulationHead {
            tag: 0,
            flag: 0,
            size: 0,
            freq_div: 0,
        };
        let _ = head.clone();

        let subseq = ModulationSubseq {
            tag: 0,
            flag: 0,
            size: 0,
        };
        let _ = subseq.clone();
    }
}
