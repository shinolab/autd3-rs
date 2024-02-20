use crate::{cpu::params::*, CPUEmulator};

#[repr(C, align(2))]
struct PhaseFilter {
    tag: u8,
}

impl CPUEmulator {
    pub(crate) unsafe fn write_phase_filter(&mut self, data: &[u8]) -> u8 {
        let data = unsafe {
            std::slice::from_raw_parts(
                data[std::mem::size_of::<PhaseFilter>()..].as_ptr() as *const u16,
                (self.num_transducers + 1) >> 1,
            )
        };

        (0..(self.num_transducers + 1) >> 1).for_each(|i| {
            self.bram_write(
                BRAM_SELECT_CONTROLLER,
                ((BRAM_CNT_SEL_FILTER as u16) << 8) | i as u16,
                data[i],
            )
        });

        NO_ERR
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_filter_memory_layout() {
        assert_eq!(2, std::mem::size_of::<PhaseFilter>());
        assert_eq!(0, memoffset::offset_of!(PhaseFilter, tag));
    }
}
