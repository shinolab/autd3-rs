use crate::{cpu::params::*, CPUEmulator};

#[repr(C, align(2))]
struct ConfigSilencer {
    tag: u8,
    flag: u8,
    value_intensity: u16,
    value_phase: u16,
}

impl CPUEmulator {
    pub(crate) fn validate_silencer_settings(&self, stm_segment: u8, mod_segment: u8) -> bool {
        if self.silencer_strict_mode {
            if self.mod_freq_div[mod_segment as usize] < self.min_freq_div_intensity
                || self.stm_freq_div[stm_segment as usize] < self.min_freq_div_intensity
                || self.stm_freq_div[stm_segment as usize] < self.min_freq_div_phase
            {
                return true;
            }
        }
        false
    }

    pub(crate) fn config_silencer(&mut self, data: &[u8]) -> u8 {
        let d = Self::cast::<ConfigSilencer>(data);

        if (d.flag & SILNCER_FLAG_MODE) == SILNCER_MODE_FIXED_COMPLETION_STEPS as _ {
            self.silencer_strict_mode = (d.flag & SILNCER_FLAG_STRICT_MODE) != 0;
            self.min_freq_div_intensity = (d.value_intensity as u32) << 9;
            self.min_freq_div_phase = (d.value_phase as u32) << 9;

            if self.validate_silencer_settings(self.stm_segment, self.mod_segment) {
                return ERR_INVALID_SILENCER_SETTING;
            }

            self.bram_write(
                BRAM_SELECT_CONTROLLER,
                ADDR_SILENCER_COMPLETION_STEPS_INTENSITY,
                d.value_intensity,
            );
            self.bram_write(
                BRAM_SELECT_CONTROLLER,
                ADDR_SILENCER_COMPLETION_STEPS_PHASE,
                d.value_phase,
            );
            self.bram_write(
                BRAM_SELECT_CONTROLLER,
                ADDR_SILENCER_MODE,
                SILNCER_MODE_FIXED_COMPLETION_STEPS as _,
            );
        } else {
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
            self.bram_write(
                BRAM_SELECT_CONTROLLER,
                ADDR_SILENCER_MODE,
                SILNCER_MODE_FIXED_UPDATE_RATE as _,
            );
        }

        NO_ERR
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silencer_memory_layout() {
        assert_eq!(6, std::mem::size_of::<ConfigSilencer>());
        assert_eq!(0, std::mem::offset_of!(ConfigSilencer, tag));
        assert_eq!(1, std::mem::offset_of!(ConfigSilencer, flag));
        assert_eq!(2, std::mem::offset_of!(ConfigSilencer, value_intensity));
        assert_eq!(4, std::mem::offset_of!(ConfigSilencer, value_phase));
    }
}
