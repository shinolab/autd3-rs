use crate::{cpu::params::*, CPUEmulator};

#[repr(C, align(2))]
#[derive(Clone, Copy)]
struct ModulationHead {
    tag: u8,
    flag: u8,
    size: u16,
    freq_div: u32,
    rep: u32,
    segment: u32,
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

#[repr(C, align(2))]
#[derive(Clone, Copy)]
struct ModulationUpdate {
    tag: u8,
    segment: u8,
}

impl CPUEmulator {
    pub(crate) unsafe fn change_mod_wr_segment(&mut self, segment: u16) {
        self.bram_write(
            BRAM_SELECT_CONTROLLER,
            BRAM_ADDR_MOD_MEM_WR_SEGMENT,
            segment,
        );
    }

    pub(crate) unsafe fn write_mod(&mut self, data: &[u8]) -> u8 {
        let d = Self::cast::<Modulation>(data);

        let write = d.subseq.size as u32;

        let data = if (d.subseq.flag & MODULATION_FLAG_BEGIN) == MODULATION_FLAG_BEGIN {
            self.mod_cycle = 0;

            let freq_div = d.head.freq_div;
            let segment = d.head.segment;
            let rep = d.head.rep;

            if self.silencer_strict_mode & (freq_div < self.min_freq_div_intensity) {
                return ERR_FREQ_DIV_TOO_SMALL;
            }
            self.mod_freq_div[segment as usize] = freq_div;

            match segment {
                0 => {
                    self.bram_cpy(
                        BRAM_SELECT_CONTROLLER,
                        BRAM_ADDR_MOD_FREQ_DIV_0_0,
                        &freq_div as *const _ as _,
                        std::mem::size_of::<u32>() >> 1,
                    );
                    self.bram_cpy(
                        BRAM_SELECT_CONTROLLER,
                        BRAM_ADDR_MOD_REP_0_0,
                        &rep as *const _ as _,
                        std::mem::size_of::<u32>() >> 1,
                    );
                }
                1 => {
                    self.bram_cpy(
                        BRAM_SELECT_CONTROLLER,
                        BRAM_ADDR_MOD_FREQ_DIV_1_0,
                        &freq_div as *const _ as _,
                        std::mem::size_of::<u32>() >> 1,
                    );
                    self.bram_cpy(
                        BRAM_SELECT_CONTROLLER,
                        BRAM_ADDR_MOD_REP_1_0,
                        &rep as *const _ as _,
                        std::mem::size_of::<u32>() >> 1,
                    );
                }
                _ => return ERR_INVALID_SEGMENT,
            }
            self.mod_segment = segment as _;

            self.change_mod_wr_segment(self.mod_segment as _);

            data[std::mem::size_of::<ModulationHead>()..].as_ptr() as *const u16
        } else {
            data[std::mem::size_of::<ModulationSubseq>()..].as_ptr() as *const u16
        };

        self.bram_cpy(
            BRAM_SELECT_MOD,
            (self.mod_cycle >> 1) as u16,
            data,
            ((write + 1) >> 1) as usize,
        );
        self.mod_cycle += write;

        if (d.subseq.flag & MODULATION_FLAG_END) == MODULATION_FLAG_END {
            match self.mod_segment {
                0 => {
                    self.bram_write(
                        BRAM_SELECT_CONTROLLER,
                        BRAM_ADDR_MOD_CYCLE_0,
                        (self.mod_cycle.max(1) - 1) as _,
                    );
                }
                1 => {
                    self.bram_write(
                        BRAM_SELECT_CONTROLLER,
                        BRAM_ADDR_MOD_CYCLE_1,
                        (self.mod_cycle.max(1) - 1) as _,
                    );
                }
                _ => unreachable!(),
            }

            if (d.subseq.flag & MODULATION_FLAG_UPDATE) == MODULATION_FLAG_UPDATE {
                self.bram_write(
                    BRAM_SELECT_CONTROLLER,
                    BRAM_ADDR_MOD_REQ_RD_SEGMENT,
                    self.mod_segment as _,
                );
            }
        }

        NO_ERR
    }

    pub(crate) unsafe fn change_mod_segment(&mut self, data: &[u8]) -> u8 {
        let d = Self::cast::<ModulationUpdate>(data);

        self.bram_write(
            BRAM_SELECT_CONTROLLER,
            BRAM_ADDR_MOD_REQ_RD_SEGMENT,
            d.segment as _,
        );

        NO_ERR
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modulation_memory_layout() {
        assert_eq!(16, std::mem::size_of::<ModulationHead>());
        assert_eq!(0, std::mem::offset_of!(ModulationHead, tag));
        assert_eq!(1, std::mem::offset_of!(ModulationHead, flag));
        assert_eq!(2, std::mem::offset_of!(ModulationHead, size));
        assert_eq!(4, std::mem::offset_of!(ModulationHead, freq_div));
        assert_eq!(8, std::mem::offset_of!(ModulationHead, rep));
        assert_eq!(12, std::mem::offset_of!(ModulationHead, segment));

        assert_eq!(4, std::mem::size_of::<ModulationSubseq>());
        assert_eq!(0, std::mem::offset_of!(ModulationSubseq, tag));
        assert_eq!(1, std::mem::offset_of!(ModulationSubseq, flag));
        assert_eq!(2, std::mem::offset_of!(ModulationSubseq, size));

        assert_eq!(16, std::mem::size_of::<Modulation>());
        assert_eq!(0, std::mem::offset_of!(Modulation, head));
        assert_eq!(0, std::mem::offset_of!(Modulation, subseq));
    }
}
