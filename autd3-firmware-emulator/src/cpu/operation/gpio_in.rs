use crate::{cpu::params::*, CPUEmulator};

#[repr(C, align(2))]
struct GPIOIn {
    tag: u8,
    flag: u8,
}

impl CPUEmulator {
    pub(crate) fn emulate_gpio_in(&mut self, data: &[u8]) -> u8 {
        let d = Self::cast::<GPIOIn>(data);
        if (d.flag & GPIO_IN_FLAG_0) == GPIO_IN_FLAG_0 {
            self.fpga_flags_internal |= CTL_FLAG_GPIO_IN_0;
        } else {
            self.fpga_flags_internal &= !CTL_FLAG_GPIO_IN_0;
        }
        if (d.flag & GPIO_IN_FLAG_1) == GPIO_IN_FLAG_1 {
            self.fpga_flags_internal |= CTL_FLAG_GPIO_IN_1;
        } else {
            self.fpga_flags_internal &= !CTL_FLAG_GPIO_IN_1;
        }
        if (d.flag & GPIO_IN_FLAG_2) == GPIO_IN_FLAG_2 {
            self.fpga_flags_internal |= CTL_FLAG_GPIO_IN_2;
        } else {
            self.fpga_flags_internal &= !CTL_FLAG_GPIO_IN_2;
        }
        if (d.flag & GPIO_IN_FLAG_3) == GPIO_IN_FLAG_3 {
            self.fpga_flags_internal |= CTL_FLAG_GPIO_IN_3;
        } else {
            self.fpga_flags_internal &= !CTL_FLAG_GPIO_IN_3;
        }

        NO_ERR
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg_attr(miri, ignore)]
    fn emulate_gpio_in_memory_layout() {
        assert_eq!(2, std::mem::size_of::<GPIOIn>());
        assert_eq!(0, std::mem::offset_of!(GPIOIn, tag));
        assert_eq!(1, std::mem::offset_of!(GPIOIn, flag));
    }
}
