use crate::{cpu::params::*, CPUEmulator};

#[repr(C, align(2))]
struct ConfigureReadsFPGAState {
    tag: u8,
    value: u8,
}

impl CPUEmulator {
    pub(crate) fn configure_reads_fpga_state(&mut self, data: &[u8]) -> u8 {
        let d = Self::cast::<ConfigureReadsFPGAState>(data);

        self.read_fpga_state = d.value != 0x00;

        NO_ERR
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configure_reads_fpga_state_memory_layout() {
        assert_eq!(2, std::mem::size_of::<ConfigureReadsFPGAState>());
        assert_eq!(0, std::mem::offset_of!(ConfigureReadsFPGAState, tag));
        assert_eq!(1, std::mem::offset_of!(ConfigureReadsFPGAState, value));
    }
}
