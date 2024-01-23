use crate::{
    cpu::params::{
        BRAM_ADDR_SILENCER_COMPLETION_STEPS_INTENSITY, BRAM_ADDR_SILENCER_COMPLETION_STEPS_PHASE,
        BRAM_ADDR_SILENCER_CTL_FLAG, BRAM_ADDR_SILENCER_UPDATE_RATE_INTENSITY,
        BRAM_ADDR_SILENCER_UPDATE_RATE_PHASE, BRAM_SELECT_CONTROLLER,
        ERR_COMPLETION_STEPS_TOO_LARGE, ERR_NONE,
    },
    CPUEmulator,
};

pub const SILENCER_CTL_FLAG_FIXED_COMPLETION_STEPS: u16 = 1 << 0;
pub const SILENCER_CTL_FLAG_STRICT_MODE: u16 = 1 << 8;

#[repr(C, align(2))]
struct ConfigSilencer {
    tag: u8,
    value_intensity: u16,
    value_phase: u16,
    flag: u16,
}

impl CPUEmulator {
    pub(crate) fn config_silencer(&mut self, data: &[u8]) -> u8 {
        let d = Self::cast::<ConfigSilencer>(data);

        if (d.flag & SILENCER_CTL_FLAG_FIXED_COMPLETION_STEPS) != 0 {
            self.silencer_strict_mode = (d.flag & SILENCER_CTL_FLAG_STRICT_MODE) != 0;
            self.min_freq_div_intensity = (d.value_intensity as u32) << 9;
            self.min_freq_div_phase = (d.value_phase as u32) << 9;
            if self.silencer_strict_mode {
                if self.mod_freq_div < self.min_freq_div_intensity {
                    return ERR_COMPLETION_STEPS_TOO_LARGE;
                }
                if (self.stm_freq_div < self.min_freq_div_intensity)
                    || (self.stm_freq_div < self.min_freq_div_phase)
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
        } else {
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
        }
        self.bram_write(BRAM_SELECT_CONTROLLER, BRAM_ADDR_SILENCER_CTL_FLAG, d.flag);

        ERR_NONE
    }
}
