use super::{super::params::*, FPGAEmulator, memory::Memory};

impl FPGAEmulator {
    #[must_use]
    pub fn gpio_in(&self) -> [bool; 4] {
        [
            (self.mem.controller_bram.borrow()[ADDR_CTL_FLAG] & (1 << CTL_FLAG_BIT_GPIO_IN_0)) != 0,
            (self.mem.controller_bram.borrow()[ADDR_CTL_FLAG] & (1 << (CTL_FLAG_BIT_GPIO_IN_1)))
                != 0,
            (self.mem.controller_bram.borrow()[ADDR_CTL_FLAG] & (1 << (CTL_FLAG_BIT_GPIO_IN_2)))
                != 0,
            (self.mem.controller_bram.borrow()[ADDR_CTL_FLAG] & (1 << (CTL_FLAG_BIT_GPIO_IN_3)))
                != 0,
        ]
    }

    #[must_use]
    pub fn debug_types(&self) -> [u8; 4] {
        [
            (self.mem.controller_bram.borrow()[ADDR_DEBUG_VALUE0_3] >> 8) as _,
            (self.mem.controller_bram.borrow()[ADDR_DEBUG_VALUE1_3] >> 8) as _,
            (self.mem.controller_bram.borrow()[ADDR_DEBUG_VALUE2_3] >> 8) as _,
            (self.mem.controller_bram.borrow()[ADDR_DEBUG_VALUE3_3] >> 8) as _,
        ]
    }

    #[must_use]
    pub fn debug_values(&self) -> [u64; 4] {
        [
            Memory::read_bram_as::<u64>(
                self.mem.controller_bram.borrow().as_slice(),
                ADDR_DEBUG_VALUE0_0,
            ) & 0x00FF_FFFF_FFFF_FFFF,
            Memory::read_bram_as::<u64>(
                self.mem.controller_bram.borrow().as_slice(),
                ADDR_DEBUG_VALUE1_0,
            ) & 0x00FF_FFFF_FFFF_FFFF,
            Memory::read_bram_as::<u64>(
                self.mem.controller_bram.borrow().as_slice(),
                ADDR_DEBUG_VALUE2_0,
            ) & 0x00FF_FFFF_FFFF_FFFF,
            Memory::read_bram_as::<u64>(
                self.mem.controller_bram.borrow().as_slice(),
                ADDR_DEBUG_VALUE3_0,
            ) & 0x00FF_FFFF_FFFF_FFFF,
        ]
    }
}
