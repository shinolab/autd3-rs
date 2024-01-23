use crate::{
    cpu::params::{BRAM_SELECT_NORMAL, CTL_FLAG_OP_MODE, ERR_NONE},
    CPUEmulator,
};

#[repr(C, align(2))]
struct Gain {
    tag: u8,
}

impl CPUEmulator {
    pub(crate) fn write_gain(&mut self, data: &[u8]) -> u8 {
        let _d = Self::cast::<Gain>(data);

        self.fpga_flags_internal &= !CTL_FLAG_OP_MODE;

        let data = unsafe {
            std::slice::from_raw_parts(
                data[std::mem::size_of::<Gain>()..].as_ptr() as *const u16,
                (data.len() - 2) >> 1,
            )
        };

        (0..self.num_transducers)
            .for_each(|i| self.bram_write(BRAM_SELECT_NORMAL, i as _, data[i]));

        ERR_NONE
    }
}
