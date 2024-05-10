use crate::{cpu::params::*, CPUEmulator};

#[repr(C, align(2))]
#[derive(Clone, Copy)]
struct ModulationHead {
    tag: u8,
    flag: u8,
    size: u16,
    transition_mode: u8,
    __pad: [u8; 3],
    freq_div: u32,
    rep: u32,
    transition_value: u64,
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
    transition_mode: u8,
    __pad: [u8; 5],
    transition_value: u64,
}

impl CPUEmulator {
    pub(crate) unsafe fn mod_segment_update(&mut self, segment: u8, mode: u8, value: u64) -> u8 {
        self.bram_write(
            BRAM_SELECT_CONTROLLER,
            ADDR_MOD_REQ_RD_SEGMENT,
            segment as _,
        );
        if (mode == TRANSITION_MODE_SYS_TIME)
            && (value < self.dc_sys_time.sys_time() + SYS_TIME_TRANSITION_MARGIN)
        {
            return ERR_MISS_TRANSITION_TIME;
        }
        self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_MOD_TRANSITION_MODE, mode as _);
        self.bram_cpy(
            BRAM_SELECT_CONTROLLER,
            ADDR_MOD_TRANSITION_VALUE_0,
            &value as *const u64 as _,
            std::mem::size_of::<u64>() >> 1,
        );
        NO_ERR
    }

    pub(crate) unsafe fn change_mod_wr_segment(&mut self, segment: u16) {
        self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_MOD_MEM_WR_SEGMENT, segment);
    }

    pub(crate) unsafe fn write_mod(&mut self, data: &[u8]) -> u8 {
        let d = Self::cast::<Modulation>(data);

        let segment = if (d.head.flag & MODULATION_FLAG_SEGMENT) != 0 {
            1
        } else {
            0
        };
        let write = d.subseq.size as u32;

        let data = if (d.subseq.flag & MODULATION_FLAG_BEGIN) == MODULATION_FLAG_BEGIN {
            self.mod_cycle = 0;

            if Self::validate_transition_mode(
                self.mod_segment,
                segment,
                d.head.rep,
                d.head.transition_mode,
            ) {
                return ERR_INVALID_TRANSITION_MODE;
            }

            if Self::validate_silencer_settings(
                self.silencer_strict_mode,
                self.min_freq_div_intensity,
                self.min_freq_div_phase,
                self.stm_freq_div[self.stm_segment as usize],
                d.head.freq_div,
            ) {
                return ERR_INVALID_SILENCER_SETTING;
            }

            if d.head.transition_mode != TRANSITION_MODE_NONE {
                self.mod_segment = segment;
            }
            self.mod_rep[segment as usize] = d.head.rep;
            self.mod_freq_div[segment as usize] = d.head.freq_div;
            self.mod_transition_mode = d.head.transition_mode;
            self.mod_transition_value = d.head.transition_value;

            match segment {
                0 => {
                    self.bram_cpy(
                        BRAM_SELECT_CONTROLLER,
                        ADDR_MOD_FREQ_DIV0_0,
                        &d.head.freq_div as *const _ as _,
                        std::mem::size_of::<u32>() >> 1,
                    );
                    self.bram_cpy(
                        BRAM_SELECT_CONTROLLER,
                        ADDR_MOD_REP0_0,
                        &d.head.rep as *const _ as _,
                        std::mem::size_of::<u32>() >> 1,
                    );
                }
                1 => {
                    self.bram_cpy(
                        BRAM_SELECT_CONTROLLER,
                        ADDR_MOD_FREQ_DIV1_0,
                        &d.head.freq_div as *const _ as _,
                        std::mem::size_of::<u32>() >> 1,
                    );
                    self.bram_cpy(
                        BRAM_SELECT_CONTROLLER,
                        ADDR_MOD_REP1_0,
                        &d.head.rep as *const _ as _,
                        std::mem::size_of::<u32>() >> 1,
                    );
                }
                _ => unreachable!(),
            }

            self.change_mod_wr_segment(segment as _);

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
            match segment {
                0 => {
                    self.bram_write(
                        BRAM_SELECT_CONTROLLER,
                        ADDR_MOD_CYCLE0,
                        (self.mod_cycle.max(1) - 1) as _,
                    );
                }
                1 => {
                    self.bram_write(
                        BRAM_SELECT_CONTROLLER,
                        ADDR_MOD_CYCLE1,
                        (self.mod_cycle.max(1) - 1) as _,
                    );
                }
                _ => unreachable!(),
            }

            if (d.subseq.flag & MODULATION_FLAG_UPDATE) == MODULATION_FLAG_UPDATE {
                return self.mod_segment_update(
                    segment,
                    self.mod_transition_mode,
                    self.mod_transition_value,
                );
            }
        }

        NO_ERR
    }

    pub(crate) unsafe fn change_mod_segment(&mut self, data: &[u8]) -> u8 {
        let d = Self::cast::<ModulationUpdate>(data);

        if Self::validate_transition_mode(
            self.mod_segment,
            d.segment,
            self.mod_rep[d.segment as usize],
            d.transition_mode,
        ) {
            return ERR_INVALID_TRANSITION_MODE;
        }

        if Self::validate_silencer_settings(
            self.silencer_strict_mode,
            self.min_freq_div_intensity,
            self.min_freq_div_phase,
            self.stm_freq_div[self.stm_segment as usize],
            self.mod_freq_div[d.segment as usize],
        ) {
            return ERR_INVALID_SILENCER_SETTING;
        }

        self.mod_segment = d.segment;
        self.mod_segment_update(d.segment, d.transition_mode, d.transition_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modulation_memory_layout() {
        assert_eq!(24, std::mem::size_of::<ModulationHead>());
        assert_eq!(0, std::mem::offset_of!(ModulationHead, tag));
        assert_eq!(1, std::mem::offset_of!(ModulationHead, flag));
        assert_eq!(2, std::mem::offset_of!(ModulationHead, size));
        assert_eq!(4, std::mem::offset_of!(ModulationHead, transition_mode));
        assert_eq!(8, std::mem::offset_of!(ModulationHead, freq_div));
        assert_eq!(12, std::mem::offset_of!(ModulationHead, rep));
        assert_eq!(16, std::mem::offset_of!(ModulationHead, transition_value));

        assert_eq!(4, std::mem::size_of::<ModulationSubseq>());
        assert_eq!(0, std::mem::offset_of!(ModulationSubseq, tag));
        assert_eq!(1, std::mem::offset_of!(ModulationSubseq, flag));
        assert_eq!(2, std::mem::offset_of!(ModulationSubseq, size));

        assert_eq!(24, std::mem::size_of::<Modulation>());
        assert_eq!(0, std::mem::offset_of!(Modulation, head));
        assert_eq!(0, std::mem::offset_of!(Modulation, subseq));

        assert_eq!(16, std::mem::size_of::<ModulationUpdate>());
        assert_eq!(0, std::mem::offset_of!(ModulationUpdate, tag));
        assert_eq!(1, std::mem::offset_of!(ModulationUpdate, segment));
        assert_eq!(2, std::mem::offset_of!(ModulationUpdate, transition_mode));
        assert_eq!(8, std::mem::offset_of!(ModulationUpdate, transition_value));
    }
}
