use crate::{CPUEmulator, cpu::params::*};

#[repr(C, align(2))]
struct CpuGPIOOut {
    tag: u8,
    pa_podr: u8,
}

impl CPUEmulator {
    pub(crate) fn cpu_gpio_out(&mut self, data: &[u8]) -> u8 {
        let d = Self::cast::<CpuGPIOOut>(data);

        self.port_a_podr = d.pa_podr;

        NO_ERR
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_out_idx_memory_layout() {
        assert_eq!(2, std::mem::size_of::<CpuGPIOOut>());
        assert_eq!(0, std::mem::offset_of!(CpuGPIOOut, tag));
        assert_eq!(1, std::mem::offset_of!(CpuGPIOOut, pa_podr));
    }
}
