use super::{super::params::*, FPGAEmulator};

impl FPGAEmulator {
    pub fn gpio_in(&self) -> [bool; 4] {
        [
            (self.mem.controller_bram()[ADDR_CTL_FLAG] & (1 << CTL_FLAG_BIT_GPIO_IN_0)) != 0,
            (self.mem.controller_bram()[ADDR_CTL_FLAG] & (1 << (CTL_FLAG_BIT_GPIO_IN_1))) != 0,
            (self.mem.controller_bram()[ADDR_CTL_FLAG] & (1 << (CTL_FLAG_BIT_GPIO_IN_2))) != 0,
            (self.mem.controller_bram()[ADDR_CTL_FLAG] & (1 << (CTL_FLAG_BIT_GPIO_IN_3))) != 0,
        ]
    }

    pub fn debug_types(&self) -> [u8; 4] {
        [
            self.mem.controller_bram()[ADDR_DEBUG_TYPE0] as _,
            self.mem.controller_bram()[ADDR_DEBUG_TYPE1] as _,
            self.mem.controller_bram()[ADDR_DEBUG_TYPE2] as _,
            self.mem.controller_bram()[ADDR_DEBUG_TYPE3] as _,
        ]
    }

    pub fn debug_values(&self) -> [u16; 4] {
        [
            self.mem.controller_bram()[ADDR_DEBUG_VALUE0],
            self.mem.controller_bram()[ADDR_DEBUG_VALUE1],
            self.mem.controller_bram()[ADDR_DEBUG_VALUE2],
            self.mem.controller_bram()[ADDR_DEBUG_VALUE3],
        ]
    }
}
