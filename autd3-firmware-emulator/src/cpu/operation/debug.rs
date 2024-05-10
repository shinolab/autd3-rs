use crate::{cpu::params::*, CPUEmulator};

#[repr(C, align(2))]
struct DebugOutIdx {
    tag: u8,
    __pad: u8,
    ty: [u8; 4],
    value: [u16; 4],
}

impl CPUEmulator {
    pub(crate) fn config_debug(&mut self, data: &[u8]) -> u8 {
        let d = Self::cast::<DebugOutIdx>(data);
        self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_DEBUG_TYPE0, d.ty[0] as _);
        self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_DEBUG_TYPE1, d.ty[1] as _);
        self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_DEBUG_TYPE2, d.ty[2] as _);
        self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_DEBUG_TYPE3, d.ty[3] as _);
        self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_DEBUG_VALUE0, d.value[0]);
        self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_DEBUG_VALUE1, d.value[1]);
        self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_DEBUG_VALUE2, d.value[2]);
        self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_DEBUG_VALUE3, d.value[3]);

        NO_ERR
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_out_idx_memory_layout() {
        assert_eq!(14, std::mem::size_of::<DebugOutIdx>());
        assert_eq!(0, std::mem::offset_of!(DebugOutIdx, tag));
        assert_eq!(2, std::mem::offset_of!(DebugOutIdx, ty));
        assert_eq!(6, std::mem::offset_of!(DebugOutIdx, value));
    }
}
