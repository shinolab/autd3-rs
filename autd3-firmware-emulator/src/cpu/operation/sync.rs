/*
 * File: sync.rs
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

use crate::{cpu::params::ERR_NONE, CPUEmulator};

#[repr(C, align(2))]
struct Sync {
    tag: u8,
}

impl CPUEmulator {
    pub(crate) fn synchronize(&mut self, data: &[u8]) -> u8 {
        let _d = Self::cast::<Sync>(data);

        self.synchronized = true;

        // Do nothing to sync

        ERR_NONE
    }
}
