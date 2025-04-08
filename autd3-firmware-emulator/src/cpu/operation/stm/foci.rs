use crate::{CPUEmulator, cpu::params::*};

pub const FOCI_STM_BUF_PAGE_SIZE_WIDTH: u16 = 12;
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
    #[must_use]
    pub(crate) unsafe fn write_foci_stm(&mut self, data: &[u8]) -> u8 {
        unsafe {
            let d = Self::cast::<FociSTM>(data);

            let segment = d.subseq.segment;

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
                self.stm_write = 0;
                self.stm_rep[segment as usize] = d.head.rep;
                self.stm_transition_mode = d.head.transition_mode;
                self.stm_transition_value = d.head.transition_value;
                self.stm_freq_div[segment as usize] = d.head.freq_div;
                self.num_foci = d.head.num_foci;

                self.bram_write(
                    BRAM_SELECT_CONTROLLER,
                    ADDR_STM_FREQ_DIV0 + segment as u16,
                    d.head.freq_div,
                );
                self.bram_write(
                    BRAM_SELECT_CONTROLLER,
                    ADDR_STM_MODE0 + segment as u16,
                    STM_MODE_FOCUS,
                );
                self.bram_write(
                    BRAM_SELECT_CONTROLLER,
                    ADDR_STM_SOUND_SPEED0 + segment as u16,
                    d.head.sound_speed,
                );
                self.bram_write(
                    BRAM_SELECT_CONTROLLER,
                    ADDR_STM_REP0 + segment as u16,
                    d.head.rep,
                );
                self.bram_write(
                    BRAM_SELECT_CONTROLLER,
                    ADDR_STM_NUM_FOCI0 + segment as u16,
                    d.head.num_foci as _,
                );

                self.change_stm_wr_segment(segment as _);
                self.change_stm_wr_page(0);

                data.as_ptr().add(std::mem::size_of::<FociSTMHead>()) as *const u16
            } else {
                data.as_ptr().add(std::mem::size_of::<FociSTMSubseq>()) as *const u16
            };

            let page_capacity =
                FOCI_STM_BUF_PAGE_SIZE - ((self.stm_write as u16) & FOCI_STM_BUF_PAGE_SIZE_MASK);
            let size = d.subseq.send_num as u16 * self.num_foci as u16;
            if size < page_capacity {
                let dst = ((self.stm_write as u16) & FOCI_STM_BUF_PAGE_SIZE_MASK) << 2;
                self.bram_cpy(BRAM_SELECT_STM, dst, src, size as usize * 4);
                self.stm_write += size as u32;
            } else {
                let dst = ((self.stm_write as u16) & FOCI_STM_BUF_PAGE_SIZE_MASK) << 2;
                src = self.bram_cpy(BRAM_SELECT_STM, dst, src, (page_capacity as usize) * 4);
                self.stm_write += page_capacity as u32;

                self.change_stm_wr_page(
                    (((self.stm_write as u16) & !FOCI_STM_BUF_PAGE_SIZE_MASK)
                        >> FOCI_STM_BUF_PAGE_SIZE_WIDTH) as _,
                );

                self.bram_cpy(BRAM_SELECT_STM, 0, src, (size - page_capacity) as usize * 4);
                self.stm_write += (size - page_capacity) as u32;
            }

            if (d.subseq.flag & FOCI_STM_FLAG_END) == FOCI_STM_FLAG_END {
                self.stm_mode[segment as usize] = STM_MODE_FOCUS;

                self.stm_cycle[segment as usize] = self.stm_write / self.num_foci as u32;

                self.bram_write(
                    BRAM_SELECT_CONTROLLER,
                    ADDR_STM_CYCLE0 + segment as u16,
                    (self.stm_cycle[segment as usize].max(1) - 1) as _,
                );

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
    }

    #[must_use]
    pub(crate) unsafe fn change_foci_stm_segment(&mut self, data: &[u8]) -> u8 {
        unsafe {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
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
