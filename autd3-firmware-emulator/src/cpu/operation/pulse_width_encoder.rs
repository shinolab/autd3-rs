use crate::{cpu::params::*, CPUEmulator};

#[repr(C, align(2))]
#[derive(Clone, Copy)]
struct PWEHead {
    tag: u8,
    flag: u8,
    size: u16,
    full_width_start: u16,
}

#[repr(C, align(2))]
#[derive(Clone, Copy)]
struct PWESubseq {
    tag: u8,
    flag: u8,
    size: u16,
}

#[repr(C, align(2))]
union PWE {
    head: PWEHead,
    subseq: PWESubseq,
}

impl CPUEmulator {
    pub(crate) unsafe fn config_pwe(&mut self, data: &[u8]) -> u8 {
        let d = Self::cast::<PWE>(data);

        let size = d.subseq.size as u32;
        if size % 2 != 0 {
            return ERR_INVALID_PWE_DATA_SIZE;
        }

        let data =
            if (d.subseq.flag & PULSE_WIDTH_ENCODER_FLAG_BEGIN) == PULSE_WIDTH_ENCODER_FLAG_BEGIN {
                self.pwe_write = 0;

                self.bram_write(
                    BRAM_SELECT_CONTROLLER,
                    ADDR_PULSE_WIDTH_ENCODER_FULL_WIDTH_START,
                    d.head.full_width_start,
                );

                data[std::mem::size_of::<PWEHead>()..].as_ptr() as *const u16
            } else {
                data[std::mem::size_of::<PWESubseq>()..].as_ptr() as *const u16
            };

        self.bram_cpy(
            BRAM_SELECT_DUTY_TABLE,
            (self.pwe_write >> 1) as u16,
            data,
            (size >> 1) as usize,
        );
        self.pwe_write += size;

        if (d.subseq.flag & PULSE_WIDTH_ENCODER_FLAG_END) == PULSE_WIDTH_ENCODER_FLAG_END {
            if self.pwe_write != 32768 {
                return ERR_PWE_INCOMPLETE_DATA;
            }
            self.set_and_wait_update(CTL_FLAG_PULSE_WIDTH_ENCODER_SET);
        }

        NO_ERR
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pwe_memory_layout() {
        assert_eq!(6, std::mem::size_of::<PWEHead>());
        assert_eq!(0, std::mem::offset_of!(PWEHead, tag));
        assert_eq!(2, std::mem::offset_of!(PWEHead, size));
        assert_eq!(4, std::mem::offset_of!(PWEHead, full_width_start));

        assert_eq!(4, std::mem::size_of::<PWESubseq>());
        assert_eq!(0, std::mem::offset_of!(PWESubseq, tag));
        assert_eq!(2, std::mem::offset_of!(PWESubseq, size));

        assert_eq!(6, std::mem::size_of::<PWE>());
        assert_eq!(0, std::mem::offset_of!(PWE, head));
        assert_eq!(0, std::mem::offset_of!(PWE, subseq));
    }
}
