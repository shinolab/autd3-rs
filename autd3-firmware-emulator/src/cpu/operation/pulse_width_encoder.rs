use crate::{cpu::params::*, CPUEmulator};

pub const PWE_TABLE_PAGE_SIZE_WIDTH: u32 = 15;
pub const PWE_TABLE_PAGE_SIZE: u32 = 1 << PWE_TABLE_PAGE_SIZE_WIDTH;
pub const PWE_TABLE_PAGE_SIZE_MASK: u32 = PWE_TABLE_PAGE_SIZE - 1;

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
    pub(crate) unsafe fn change_pwe_wr_page(&mut self, page: u16) {
        self.bram_write(
            BRAM_SELECT_CONTROLLER,
            ADDR_PULSE_WIDTH_ENCODER_TABLE_WR_PAGE,
            page,
        );
    }

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

                self.change_pwe_wr_page(0);

                data[std::mem::size_of::<PWEHead>()..].as_ptr() as *const u16
            } else {
                data[std::mem::size_of::<PWESubseq>()..].as_ptr() as *const u16
            };

        let page_capacity =
            (self.pwe_write & !PWE_TABLE_PAGE_SIZE_MASK) + PWE_TABLE_PAGE_SIZE - self.pwe_write;

        if size <= page_capacity {
            self.bram_cpy(
                BRAM_SELECT_DUTY_TABLE,
                ((self.pwe_write & PWE_TABLE_PAGE_SIZE_MASK) >> 1) as u16,
                data,
                (size >> 1) as usize,
            );
            self.pwe_write += size;
        } else {
            self.bram_cpy(
                BRAM_SELECT_DUTY_TABLE,
                ((self.pwe_write & PWE_TABLE_PAGE_SIZE_MASK) >> 1) as u16,
                data,
                (page_capacity >> 1) as usize,
            );
            self.pwe_write += page_capacity;
            let data = data.add((page_capacity >> 1) as usize);
            self.change_pwe_wr_page(
                ((self.pwe_write & !PWE_TABLE_PAGE_SIZE_MASK) >> PWE_TABLE_PAGE_SIZE_WIDTH) as _,
            );
            self.bram_cpy(
                BRAM_SELECT_DUTY_TABLE,
                ((self.pwe_write & PWE_TABLE_PAGE_SIZE_MASK) >> 1) as u16,
                data,
                ((size - page_capacity) >> 1) as usize,
            );
            self.pwe_write += size - page_capacity;
        }

        if ((d.subseq.flag & PULSE_WIDTH_ENCODER_FLAG_END) == PULSE_WIDTH_ENCODER_FLAG_END)
            && self.pwe_write != 65536
        {
            return ERR_PWE_INCOMPLETE_DATA;
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
