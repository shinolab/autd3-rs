/*
 * File: gain_stm_control_flags.rs
 * Project: gain
 * Created Date: 08/10/2023
 * Author: Shun Suzuki
 * -----
 * Last Modified: 09/11/2023
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2023 Shun Suzuki. All rights reserved.
 *
 */

use std::fmt;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct GainSTMControlFlags(u8);

bitflags::bitflags! {
    impl GainSTMControlFlags : u8 {
        const NONE            = 0;
        const LEGACY          = 1 << 0;
        const STM_BEGIN       = 1 << 2;
        const STM_END         = 1 << 3;
        const USE_START_IDX   = 1 << 4;
        const USE_FINISH_IDX  = 1 << 5;
        const _RESERVED_0     = 1 << 6;
        const _RESERVED_1     = 1 << 7;
    }
}

impl fmt::Display for GainSTMControlFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut flags = Vec::new();
        if self.contains(GainSTMControlFlags::STM_BEGIN) {
            flags.push("STM_BEGIN")
        }
        if self.contains(GainSTMControlFlags::STM_END) {
            flags.push("STM_END")
        }
        if self.contains(GainSTMControlFlags::USE_START_IDX) {
            flags.push("USE_START_IDX")
        }
        if self.contains(GainSTMControlFlags::USE_FINISH_IDX) {
            flags.push("USE_FINISH_IDX")
        }
        if self.is_empty() {
            flags.push("NONE")
        }
        write!(
            f,
            "{}",
            flags
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(" | ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gain_stm_controll_flag() {
        assert_eq!(std::mem::size_of::<GainSTMControlFlags>(), 1);

        let flags = GainSTMControlFlags::STM_BEGIN | GainSTMControlFlags::USE_START_IDX;

        let flagsc = Clone::clone(&flags);
        assert_eq!(flagsc.bits(), flags.bits());
    }

    #[test]
    fn gain_stm_controll_flag_fmt() {
        assert_eq!(format!("{}", GainSTMControlFlags::NONE), "NONE");
        assert_eq!(format!("{}", GainSTMControlFlags::STM_BEGIN), "STM_BEGIN");
        assert_eq!(format!("{}", GainSTMControlFlags::STM_END), "STM_END");
        assert_eq!(
            format!("{}", GainSTMControlFlags::USE_START_IDX),
            "USE_START_IDX"
        );
        assert_eq!(
            format!("{}", GainSTMControlFlags::USE_FINISH_IDX),
            "USE_FINISH_IDX"
        );

        assert_eq!(
            format!(
                "{}",
                GainSTMControlFlags::STM_BEGIN
                    | GainSTMControlFlags::STM_END
                    | GainSTMControlFlags::USE_START_IDX
                    | GainSTMControlFlags::USE_FINISH_IDX
            ),
            "STM_BEGIN | STM_END | USE_START_IDX | USE_FINISH_IDX"
        );
    }
}
