use std::mem::size_of;

use crate::{CPUEmulator, cpu::params::*};

#[repr(C, align(2))]
#[derive(Clone, Copy)]
struct OutputMask {
    tag: u8,
    segment: u8,
}

impl CPUEmulator {
    #[must_use]
    pub(crate) unsafe fn output_mask(&mut self, data: &[u8]) -> u8 {
        let d = Self::cast::<OutputMask>(data);

        self.bram_cpy(
            BRAM_SELECT_CONTROLLER,
            ((BRAM_CNT_SEL_OUTPUT_MASK as u16) << 8) | ((d.segment as u16) << 4),
            data[size_of::<OutputMask>()..].as_ptr() as _,
            16,
        );

        NO_ERR
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_mask_layout() {
        assert_eq!(2, std::mem::size_of::<OutputMask>());
        assert_eq!(0, std::mem::offset_of!(OutputMask, tag));
        assert_eq!(1, std::mem::offset_of!(OutputMask, segment));
    }
}
