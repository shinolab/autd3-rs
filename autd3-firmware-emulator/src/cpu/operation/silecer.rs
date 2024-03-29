use crate::{cpu::params::*, CPUEmulator};

#[repr(C, align(2))]
struct ConfigSilencer {
    tag: u8,
    flag: u8,
    value_intensity: u16,
    value_phase: u16,
}

impl CPUEmulator {
    pub(crate) fn config_silencer(&mut self, data: &[u8]) -> u8 {
        let d = Self::cast::<ConfigSilencer>(data);

        match d.flag & SILNCER_FLAG_MODE {
            v if v == SILNCER_MODE_FIXED_COMPLETION_STEPS as _ => {
                self.silencer_strict_mode = (d.flag & SILNCER_FLAG_STRICT_MODE) != 0;
                self.min_freq_div_intensity = (d.value_intensity as u32) << 9;
                self.min_freq_div_phase = (d.value_phase as u32) << 9;
                if self.silencer_strict_mode {
                    if self
                        .mod_freq_div
                        .iter()
                        .any(|&v| v < self.min_freq_div_intensity)
                    {
                        return ERR_COMPLETION_STEPS_TOO_LARGE;
                    }
                    if self
                        .stm_freq_div
                        .iter()
                        .any(|&v| v < self.min_freq_div_intensity || v < self.min_freq_div_phase)
                    {
                        return ERR_COMPLETION_STEPS_TOO_LARGE;
                    }
                }
                self.bram_write(
                    BRAM_SELECT_CONTROLLER,
                    BRAM_ADDR_SILENCER_COMPLETION_STEPS_INTENSITY,
                    d.value_intensity,
                );
                self.bram_write(
                    BRAM_SELECT_CONTROLLER,
                    BRAM_ADDR_SILENCER_COMPLETION_STEPS_PHASE,
                    d.value_phase,
                );
                self.bram_write(
                    BRAM_SELECT_CONTROLLER,
                    BRAM_ADDR_SILENCER_MODE,
                    SILNCER_MODE_FIXED_COMPLETION_STEPS as _,
                );
            }
            v if v == SILNCER_MODE_FIXED_UPDATE_RATE as _ => {
                self.bram_write(
                    BRAM_SELECT_CONTROLLER,
                    BRAM_ADDR_SILENCER_UPDATE_RATE_INTENSITY,
                    d.value_intensity,
                );
                self.bram_write(
                    BRAM_SELECT_CONTROLLER,
                    BRAM_ADDR_SILENCER_UPDATE_RATE_PHASE,
                    d.value_phase,
                );
                self.bram_write(
                    BRAM_SELECT_CONTROLLER,
                    BRAM_ADDR_SILENCER_MODE,
                    SILNCER_MODE_FIXED_UPDATE_RATE as _,
                );
            }
            _ => return ERR_INVALID_MODE,
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
