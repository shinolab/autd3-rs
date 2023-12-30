/*
 * File: mod_delay.rs
 * Project: operation
 * Created Date: 30/12/2023
 * Author: Shun Suzuki
 * -----
 * Last Modified: 30/12/2023
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2023 Shun Suzuki. All rights reserved.
 *
 */

use crate::{
    cpu::params::{BRAM_ADDR_MOD_DELAY_BASE, BRAM_SELECT_CONTROLLER, ERR_NONE},
    CPUEmulator,
};

#[repr(C, align(2))]
struct ConfigureModDelay {
    tag: u8,
}

impl CPUEmulator {
    pub(crate) fn write_mod_delay(&mut self, data: &[u8]) -> u8 {
        let _d = Self::cast::<ConfigureModDelay>(data);

        let delays = unsafe {
            std::slice::from_raw_parts(
                data[std::mem::size_of::<ConfigureModDelay>()..].as_ptr() as *const u16,
                self.num_transducers,
            )
        };
        self.bram_cpy(
            BRAM_SELECT_CONTROLLER,
            BRAM_ADDR_MOD_DELAY_BASE,
            delays.as_ptr(),
            delays.len(),
        );

        ERR_NONE
    }
}
