use std::mem::size_of;

use crate::{cpu::params::*, CPUEmulator};

#[repr(C, align(2))]
#[derive(Clone, Copy)]
struct Pwe {
    tag: u8,
}

impl CPUEmulator {
    pub(crate) unsafe fn config_pwe(&mut self, data: &[u8]) -> u8 {
        let _d = Self::cast::<Pwe>(data);

        self.bram_cpy(
            BRAM_SELECT_PWE_TABLE,
            0,
            data[size_of::<Pwe>()..].as_ptr() as _,
            (256 >> 1) as usize,
        );

        NO_ERR
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pwe_memory_layout() {
        assert_eq!(2, std::mem::size_of::<Pwe>());
        assert_eq!(0, std::mem::offset_of!(Pwe, tag));
    }
}
