use crate::{CPUEmulator, cpu::params::*};

#[repr(C, align(2))]
#[derive(Clone, Copy)]
struct Clk {
    tag: u8,
    flag: u8,
    size: u16,
}

impl CPUEmulator {
    pub(crate) unsafe fn configure_clk(&mut self, data: &[u8]) -> u8 {
        let d = Self::cast::<Clk>(data);

        let size = d.size;

        if (d.flag & CLK_FLAG_BEGIN) == CLK_FLAG_BEGIN {
            self.clk_write = 0;
        }

        self.bram_cpy(
            BRAM_SELECT_CONTROLLER,
            ((BRAM_CNT_SELECT_CLOCK as u16) << 8) | (self.clk_write << 2),
            data[std::mem::size_of::<Clk>()..].as_ptr() as *const u16,
            (size << 2) as usize,
        );
        self.clk_write += size;

        if ((d.flag & CLK_FLAG_END) == CLK_FLAG_END) && self.clk_write != 32 {
            return ERR_CLK_INCOMPLETE_DATA;
        }

        NO_ERR
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clk_memory_layout() {
        assert_eq!(4, std::mem::size_of::<Clk>());
        assert_eq!(0, std::mem::offset_of!(Clk, tag));
        assert_eq!(1, std::mem::offset_of!(Clk, flag));
        assert_eq!(2, std::mem::offset_of!(Clk, size));
    }
}
