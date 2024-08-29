use autd3_driver::firmware::fpga::SilencerTarget;

use super::{super::params::*, FPGAEmulator};

impl FPGAEmulator {
    pub fn silencer_update_rate(&self) -> (u16, u16) {
        (
            self.mem.controller_bram()[ADDR_SILENCER_UPDATE_RATE_INTENSITY],
            self.mem.controller_bram()[ADDR_SILENCER_UPDATE_RATE_PHASE],
        )
    }

    pub fn silencer_completion_steps(&self) -> (u8, u8) {
        (
            self.mem.controller_bram()[ADDR_SILENCER_COMPLETION_STEPS_INTENSITY] as _,
            self.mem.controller_bram()[ADDR_SILENCER_COMPLETION_STEPS_PHASE] as _,
        )
    }

    pub fn silencer_fixed_update_rate_mode(&self) -> bool {
        (self.mem.controller_bram()[ADDR_SILENCER_FLAG] & SILENCER_FLAG_FIXED_UPDATE_RATE_MODE)
            == SILENCER_FLAG_FIXED_UPDATE_RATE_MODE
    }

    pub fn silencer_fixed_completion_steps_mode(&self) -> bool {
        !self.silencer_fixed_update_rate_mode()
    }

    pub fn silencer_target(&self) -> SilencerTarget {
        if (self.mem.controller_bram()[ADDR_SILENCER_FLAG] & SILENCER_FLAG_PULSE_WIDTH)
            == SILENCER_FLAG_PULSE_WIDTH
        {
            SilencerTarget::PulseWidth
        } else {
            SilencerTarget::Intensity
        }
    }
}
