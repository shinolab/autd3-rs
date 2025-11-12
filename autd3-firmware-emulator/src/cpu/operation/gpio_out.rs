use crate::{CPUEmulator, cpu::params::*};

#[repr(C, align(2))]
struct GPIOOutput {
    tag: u8,
    __: [u8; 7],
    value: [u64; 4],
}

impl CPUEmulator {
    #[must_use]
    pub(crate) fn config_gpio_output(&mut self, data: &[u8]) -> u8 {
        let d = Self::cast::<GPIOOutput>(data);
        self.bram_cpy(
            BRAM_SELECT_CONTROLLER,
            ADDR_DEBUG_VALUE0_0,
            d.value.as_ptr() as _,
            4 * std::mem::size_of::<u64>() / std::mem::size_of::<u16>(),
        );

        self.set_and_wait_update(CTL_FLAG_DEBUG_SET);

        NO_ERR
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mem_layout() {
        assert_eq!(40, std::mem::size_of::<GPIOOutput>());
        assert_eq!(0, std::mem::offset_of!(GPIOOutput, tag));
        assert_eq!(8, std::mem::offset_of!(GPIOOutput, value));
    }
}
