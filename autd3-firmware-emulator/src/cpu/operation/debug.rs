use crate::{
    cpu::params::{BRAM_ADDR_DEBUG_OUT_IDX, BRAM_SELECT_CONTROLLER, ERR_NONE},
    CPUEmulator,
};

#[repr(C, align(2))]
struct DebugOutIdx {
    tag: u8,
    idx: u8,
}

impl CPUEmulator {
    pub(crate) fn config_debug(&mut self, data: &[u8]) -> u8 {
        let d = Self::cast::<DebugOutIdx>(data);
        self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_DEBUG_OUT_IDX, d.idx as _);

        ERR_NONE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_out_idx_memory_layout() {
        assert_eq!(2, std::mem::size_of::<DebugOutIdx>());
        assert_eq!(0, memoffset::offset_of!(DebugOutIdx, tag));
        assert_eq!(1, memoffset::offset_of!(DebugOutIdx, idx));
    }
}
