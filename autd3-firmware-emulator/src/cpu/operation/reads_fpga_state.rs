/*
 * File: reads_fpga_state.rs
 * Project: operation
 * Created Date: 30/12/2023
 * Author: Shun Suzuki
 * -----
 * Last Modified: 17/01/2024
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2023 Shun Suzuki. All rights reserved.
 *
 */

use crate::{cpu::params::ERR_NONE, CPUEmulator};

#[repr(C, align(2))]
struct ConfigureReadsFPGAState {
    tag: u8,
    value: u8,
}

impl CPUEmulator {
    pub(crate) fn configure_reads_fpga_state(&mut self, data: &[u8]) -> u8 {
        let d = Self::cast::<ConfigureReadsFPGAState>(data);

        self.read_fpga_state = d.value != 0x00;

        ERR_NONE
    }
}
