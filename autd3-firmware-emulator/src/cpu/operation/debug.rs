/*
 * File: debug.rs
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
