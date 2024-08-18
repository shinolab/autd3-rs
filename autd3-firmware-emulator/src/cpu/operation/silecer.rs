use crate::{cpu::params::*, CPUEmulator};

#[repr(C, align(2))]
struct ConfigSilencer {
    tag: u8,
    flag: u8,
    value_intensity: u8,
    value_phase: u8,
}

#[repr(C, align(2))]
struct ConfigSilencer2 {
    tag: u8,
    flag: u8,
    value_intensity: u16,
    value_phase: u16,
}

impl CPUEmulator {
    pub(crate) fn validate_silencer_settings(&self, stm_freq_div: u16, mod_freq_div: u16) -> bool {
        if self.silencer_strict_mode
            && (mod_freq_div < self.min_freq_div_intensity
                || stm_freq_div < self.min_freq_div_intensity
                || stm_freq_div < self.min_freq_div_phase)
        {
            return true;
        }
        false
    }

    // GRCOV_EXCL_START
    pub(crate) fn config_silencer(&mut self, data: &[u8]) -> u8 {
        let d = Self::cast::<ConfigSilencer>(data);

        if (d.flag & SILENCER_FLAG_FIXED_UPDATE_RATE_MODE)
            == SILENCER_FLAG_FIXED_UPDATE_RATE_MODE as _
        {
            self.bram_write(
                BRAM_SELECT_CONTROLLER,
                ADDR_SILENCER_UPDATE_RATE_INTENSITY,
                (d.value_intensity as u16) << 8,
            );
            self.bram_write(
                BRAM_SELECT_CONTROLLER,
                ADDR_SILENCER_UPDATE_RATE_PHASE,
                (d.value_phase as u16) << 8,
            );
        } else {
            let strict_mode = self.silencer_strict_mode;
            let min_freq_div_intensity = self.min_freq_div_intensity;
            let min_freq_div_phase = self.min_freq_div_phase;

            self.silencer_strict_mode =
                (d.flag & SILENCER_FLAG_STRICT_MODE) == SILENCER_FLAG_STRICT_MODE;
            self.min_freq_div_intensity = d.value_intensity as _;
            self.min_freq_div_phase = d.value_phase as _;

            if self.validate_silencer_settings(
                self.stm_freq_div[self.stm_segment as usize],
                self.mod_freq_div[self.mod_segment as usize],
            ) {
                self.silencer_strict_mode = strict_mode;
                self.min_freq_div_intensity = min_freq_div_intensity;
                self.min_freq_div_phase = min_freq_div_phase;
                return ERR_INVALID_SILENCER_SETTING;
            }

            self.bram_write(
                BRAM_SELECT_CONTROLLER,
                ADDR_SILENCER_COMPLETION_STEPS_INTENSITY,
                d.value_intensity as _,
            );
            self.bram_write(
                BRAM_SELECT_CONTROLLER,
                ADDR_SILENCER_COMPLETION_STEPS_PHASE,
                d.value_phase as _,
            );
        }
        self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_SILENCER_FLAG, d.flag as _);

        self.set_and_wait_update(CTL_FLAG_SILENCER_SET);

        NO_ERR
    }
    // GRCOV_EXCL_STOP

    pub(crate) fn config_silencer2(&mut self, data: &[u8]) -> u8 {
        let d = Self::cast::<ConfigSilencer2>(data);

        if (d.flag & SILENCER_FLAG_FIXED_UPDATE_RATE_MODE)
            == SILENCER_FLAG_FIXED_UPDATE_RATE_MODE as _
        {
            self.bram_write(
                BRAM_SELECT_CONTROLLER,
                ADDR_SILENCER_UPDATE_RATE_INTENSITY,
                d.value_intensity,
            );
            self.bram_write(
                BRAM_SELECT_CONTROLLER,
                ADDR_SILENCER_UPDATE_RATE_PHASE,
                d.value_phase,
            );
        } else {
            let strict_mode = self.silencer_strict_mode;
            let min_freq_div_intensity = self.min_freq_div_intensity;
            let min_freq_div_phase = self.min_freq_div_phase;

            self.silencer_strict_mode =
                (d.flag & SILENCER_FLAG_STRICT_MODE) == SILENCER_FLAG_STRICT_MODE;
            self.min_freq_div_intensity = d.value_intensity & 0x00FF;
            self.min_freq_div_phase = d.value_phase & 0x00FF;

            if self.validate_silencer_settings(
                self.stm_freq_div[self.stm_segment as usize],
                self.mod_freq_div[self.mod_segment as usize],
            ) {
                self.silencer_strict_mode = strict_mode;
                self.min_freq_div_intensity = min_freq_div_intensity;
                self.min_freq_div_phase = min_freq_div_phase;
                return ERR_INVALID_SILENCER_SETTING;
            }

            self.bram_write(
                BRAM_SELECT_CONTROLLER,
                ADDR_SILENCER_COMPLETION_STEPS_INTENSITY,
                d.value_intensity & 0x00FF,
            );
            self.bram_write(
                BRAM_SELECT_CONTROLLER,
                ADDR_SILENCER_COMPLETION_STEPS_PHASE,
                d.value_phase & 0x00FF,
            );
        }
        self.bram_write(BRAM_SELECT_CONTROLLER, ADDR_SILENCER_FLAG, d.flag as _);

        self.set_and_wait_update(CTL_FLAG_SILENCER_SET);

        NO_ERR
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg_attr(miri, ignore)]
    fn silencer_memory_layout() {
        assert_eq!(6, std::mem::size_of::<ConfigSilencer2>());
        assert_eq!(0, std::mem::offset_of!(ConfigSilencer2, tag));
        assert_eq!(1, std::mem::offset_of!(ConfigSilencer2, flag));
        assert_eq!(2, std::mem::offset_of!(ConfigSilencer2, value_intensity));
        assert_eq!(4, std::mem::offset_of!(ConfigSilencer2, value_phase));
    }
}
