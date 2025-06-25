use crate::{CPUEmulator, cpu::params::*};

#[repr(C, align(2))]
struct ConfigSilencer {
    tag: u8,
    flag: u8,
    value_intensity: u16,
    value_phase: u16,
}

impl CPUEmulator {
    #[must_use]
    pub(crate) const fn validate_silencer_settings(
        &self,
        stm_freq_div: u16,
        mod_freq_div: u16,
    ) -> bool {
        if self.silencer_strict
            && (mod_freq_div < self.min_freq_div_intensity
                || stm_freq_div < self.min_freq_div_intensity
                || stm_freq_div < self.min_freq_div_phase)
        {
            return true;
        }
        false
    }

    #[must_use]
    pub(crate) fn config_silencer(&mut self, data: &[u8]) -> u8 {
        let d = Self::cast::<ConfigSilencer>(data);

        if (d.flag & SILENCER_FLAG_FIXED_UPDATE_RATE_MODE) == SILENCER_FLAG_FIXED_UPDATE_RATE_MODE {
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
            let strict = self.silencer_strict;
            let min_freq_div_intensity = self.min_freq_div_intensity;
            let min_freq_div_phase = self.min_freq_div_phase;

            self.silencer_strict =
                (d.flag & SILENCER_FLAG_STRICT_MODE) == SILENCER_FLAG_STRICT_MODE;
            self.min_freq_div_intensity = d.value_intensity;
            self.min_freq_div_phase = d.value_phase;

            if self.validate_silencer_settings(
                self.stm_freq_div[self.stm_segment as usize],
                self.mod_freq_div[self.mod_segment as usize],
            ) {
                self.silencer_strict = strict;
                self.min_freq_div_intensity = min_freq_div_intensity;
                self.min_freq_div_phase = min_freq_div_phase;
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
    fn silencer_memory_layout() {
        assert_eq!(6, std::mem::size_of::<ConfigSilencer>());
        assert_eq!(0, std::mem::offset_of!(ConfigSilencer, tag));
        assert_eq!(1, std::mem::offset_of!(ConfigSilencer, flag));
        assert_eq!(2, std::mem::offset_of!(ConfigSilencer, value_intensity));
        assert_eq!(4, std::mem::offset_of!(ConfigSilencer, value_phase));
    }
}
