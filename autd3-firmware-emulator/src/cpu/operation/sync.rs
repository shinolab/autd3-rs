use autd3_driver::defined::Hz;

use crate::{cpu::params::*, CPUEmulator};

#[repr(C, align(2))]
struct Sync {
    tag: u8,
    __pad: [u8; 3],
    ecat_sync_base_cnt: u32,
}

impl CPUEmulator {
    pub(crate) fn synchronize(&mut self, data: &[u8]) -> u8 {
        let d = Self::cast::<Sync>(data);

        self.synchronized = true;
        self.set_and_wait_update(CTL_FLAG_SYNC_SET);

        self.fpga
            .set_fpga_clk_freq(d.ecat_sync_base_cnt * 2000 * Hz);

        NO_ERR
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_memory_layout() {
        assert_eq!(8, std::mem::size_of::<Sync>());
        assert_eq!(0, std::mem::offset_of!(Sync, tag));
        assert_eq!(4, std::mem::offset_of!(Sync, ecat_sync_base_cnt));
    }
}
