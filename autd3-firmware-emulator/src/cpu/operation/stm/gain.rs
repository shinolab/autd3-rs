use crate::{
    cpu::params::{
        BRAM_ADDR_STM_ADDR_OFFSET, BRAM_ADDR_STM_CYCLE, BRAM_ADDR_STM_FINISH_IDX,
        BRAM_ADDR_STM_FREQ_DIV_0, BRAM_ADDR_STM_START_IDX, BRAM_SELECT_CONTROLLER, BRAM_SELECT_STM,
        CTL_FLAG_OP_MODE, CTL_FLAG_USE_STM_FINISH_IDX, CTL_FLAG_USE_STM_START_IDX,
        CTL_REG_STM_GAIN_MODE, ERR_FREQ_DIV_TOO_SMALL, ERR_NONE,
    },
    CPUEmulator,
};

const GAIN_STM_FLAG_BEGIN: u8 = 1 << 2;
const GAIN_STM_FLAG_END: u8 = 1 << 3;
const GAIN_STM_FLAG_USE_START_IDX: u8 = 1 << 4;
const GAIN_STM_FLAG_USE_FINISH_IDX: u8 = 1 << 5;

const GAIN_STM_MODE_INTENSITY_PHASE_FULL: u16 = 0;
const GAIN_STM_MODE_PHASE_FULL: u16 = 1;
const GAIN_STM_MODE_PHASE_HALF: u16 = 2;

pub const GAIN_STM_BUF_PAGE_SIZE_WIDTH: u32 = 6;
pub const GAIN_STM_BUF_PAGE_SIZE: u32 = 1 << GAIN_STM_BUF_PAGE_SIZE_WIDTH;
pub const GAIN_STM_BUF_PAGE_SIZE_MASK: u32 = GAIN_STM_BUF_PAGE_SIZE - 1;

#[repr(C)]
#[derive(Clone, Copy)]
struct GainSTMHead {
    tag: u8,
    flag: u8,
    mode: u16,
    freq_div: u32,
    start_idx: u16,
    finish_idx: u16,
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

impl CPUEmulator {
    pub(crate) unsafe fn write_gain_stm(&mut self, data: &[u8]) -> u8 {
        let d = Self::cast::<GainSTM>(data);

        let send = (d.subseq.flag >> 6) + 1;
        let src_base = if (d.subseq.flag & GAIN_STM_FLAG_BEGIN) == GAIN_STM_FLAG_BEGIN {
            self.stm_cycle = 0;
            self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_STM_ADDR_OFFSET, 0);

            self.gain_stm_mode = d.head.mode;

            let freq_div = d.head.freq_div;
            if self.silencer_strict_mode
                && ((freq_div < self.min_freq_div_intensity)
                    || (freq_div < self.min_freq_div_phase))
            {
                return ERR_FREQ_DIV_TOO_SMALL;
            }
            self.stm_freq_div = freq_div;
            self.bram_cpy(
                BRAM_SELECT_CONTROLLER,
                BRAM_ADDR_STM_FREQ_DIV_0,
                &freq_div as *const _ as _,
                std::mem::size_of::<u32>() >> 1,
            );

            let start_idx = d.head.start_idx;
            let finish_idx = d.head.finish_idx;

            self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_STM_START_IDX, start_idx);
            self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_STM_FINISH_IDX, finish_idx);

            if (d.head.flag & GAIN_STM_FLAG_USE_START_IDX) == GAIN_STM_FLAG_USE_START_IDX {
                self.fpga_flags_internal |= CTL_FLAG_USE_STM_START_IDX;
            } else {
                self.fpga_flags_internal &= !CTL_FLAG_USE_STM_START_IDX;
            }
            if (d.head.flag & GAIN_STM_FLAG_USE_FINISH_IDX) == GAIN_STM_FLAG_USE_FINISH_IDX {
                self.fpga_flags_internal |= CTL_FLAG_USE_STM_FINISH_IDX;
            } else {
                self.fpga_flags_internal &= !CTL_FLAG_USE_STM_FINISH_IDX;
            }

            unsafe { data.as_ptr().add(12) as *const u16 }
        } else {
            unsafe { data.as_ptr().add(2) as *const u16 }
        };

        let mut src = src_base;
        let mut dst = ((self.stm_cycle & GAIN_STM_BUF_PAGE_SIZE_MASK) << 8) as u16;

        if self.gain_stm_mode == GAIN_STM_MODE_INTENSITY_PHASE_FULL {
            self.stm_cycle += 1;
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
            self.stm_cycle += 1;

            if send > 1 {
                let mut src = src_base;
                let mut dst = ((self.stm_cycle & GAIN_STM_BUF_PAGE_SIZE_MASK) << 8) as u16;
                (0..self.num_transducers).for_each(|_| unsafe {
                    self.bram_write(BRAM_SELECT_STM, dst, 0xFF00 | ((src.read() >> 8) & 0x00FF));
                    dst += 1;
                    src = src.add(1);
                });
                self.stm_cycle += 1;
            }
        } else if self.gain_stm_mode == GAIN_STM_MODE_PHASE_HALF {
            (0..self.num_transducers).for_each(|_| unsafe {
                let phase = src.read() & 0x000F;
                self.bram_write(BRAM_SELECT_STM, dst, 0xFF00 | (phase << 4) | phase);
                dst += 1;
                src = src.add(1);
            });
            self.stm_cycle += 1;

            if send > 1 {
                let mut src = src_base;
                let mut dst = ((self.stm_cycle & GAIN_STM_BUF_PAGE_SIZE_MASK) << 8) as u16;
                (0..self.num_transducers).for_each(|_| unsafe {
                    let phase = (src.read() >> 4) & 0x000F;
                    self.bram_write(BRAM_SELECT_STM, dst, 0xFF00 | (phase << 4) | phase);
                    dst += 1;
                    src = src.add(1);
                });
                self.stm_cycle += 1;
            }

            if send > 2 {
                let mut src = src_base;
                let mut dst = ((self.stm_cycle & GAIN_STM_BUF_PAGE_SIZE_MASK) << 8) as u16;
                (0..self.num_transducers).for_each(|_| unsafe {
                    let phase = (src.read() >> 8) & 0x000F;
                    self.bram_write(BRAM_SELECT_STM, dst, 0xFF00 | (phase << 4) | phase);
                    dst += 1;
                    src = src.add(1);
                });
                self.stm_cycle += 1;
            }

            if send > 3 {
                let mut src = src_base;
                let mut dst = ((self.stm_cycle & GAIN_STM_BUF_PAGE_SIZE_MASK) << 8) as u16;
                (0..self.num_transducers).for_each(|_| unsafe {
                    let phase = (src.read() >> 12) & 0x000F;
                    self.bram_write(BRAM_SELECT_STM, dst, 0xFF00 | (phase << 4) | phase);
                    dst += 1;
                    src = src.add(1);
                });
                self.stm_cycle += 1;
            }
        }

        if self.stm_cycle & GAIN_STM_BUF_PAGE_SIZE_MASK == 0 {
            self.bram_write(
                BRAM_SELECT_CONTROLLER,
                BRAM_ADDR_STM_ADDR_OFFSET,
                ((self.stm_cycle & !GAIN_STM_BUF_PAGE_SIZE_MASK) >> GAIN_STM_BUF_PAGE_SIZE_WIDTH)
                    as _,
            );
        }

        if (d.subseq.flag & GAIN_STM_FLAG_END) == GAIN_STM_FLAG_END {
            self.fpga_flags_internal |= CTL_FLAG_OP_MODE;
            self.fpga_flags_internal |= CTL_REG_STM_GAIN_MODE;
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
    fn gain_stm_derive() {
        assert_eq!(std::mem::size_of::<GainSTMHead>(), 12);
        assert_eq!(memoffset::offset_of!(GainSTMHead, tag), 0);
        assert_eq!(memoffset::offset_of!(GainSTMHead, flag), 1);
        assert_eq!(memoffset::offset_of!(GainSTMHead, mode), 2);
        assert_eq!(memoffset::offset_of!(GainSTMHead, freq_div), 4);
        assert_eq!(memoffset::offset_of!(GainSTMHead, start_idx), 8);
        assert_eq!(memoffset::offset_of!(GainSTMHead, finish_idx), 10);

        assert_eq!(std::mem::size_of::<GainSTMSubseq>(), 2);
        assert_eq!(memoffset::offset_of!(GainSTMSubseq, tag), 0);
        assert_eq!(memoffset::offset_of!(GainSTMSubseq, flag), 1);

        assert_eq!(memoffset::offset_of_union!(GainSTM, head), 0);
        assert_eq!(memoffset::offset_of_union!(GainSTM, subseq), 0);

        let head = GainSTMHead {
            tag: 0,
            flag: 0,
            mode: 0,
            freq_div: 0,
            start_idx: 0,
            finish_idx: 0,
        };
        let _ = head.clone();

        let subseq = GainSTMSubseq { tag: 0, flag: 0 };
        let _ = subseq.clone();
    }
}
