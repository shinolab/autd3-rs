use std::mem::size_of;

use crate::{CPUEmulator, cpu::params::*};

#[repr(C, align(2))]
#[derive(Clone, Copy)]
struct PhaseCorr {
    tag: u8,
}

impl CPUEmulator {
    pub(crate) unsafe fn phase_corr(&mut self, data: &[u8]) -> u8 {
        let _d = Self::cast::<PhaseCorr>(data);

        self.bram_cpy(
            BRAM_SELECT_CONTROLLER,
            (BRAM_CNT_SEL_PHASE_CORR as u16) << 8,
            data[size_of::<PhaseCorr>()..].as_ptr() as _,
            (TRANS_NUM + 1) >> 1,
        );

        NO_ERR
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pwe_memory_layout() {
        assert_eq!(2, std::mem::size_of::<PhaseCorr>());
        assert_eq!(0, std::mem::offset_of!(PhaseCorr, tag));
    }
}
