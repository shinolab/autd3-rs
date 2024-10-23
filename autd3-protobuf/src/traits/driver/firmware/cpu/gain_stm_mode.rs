use crate::pb::*;

impl From<GainStmMode> for autd3_driver::firmware::cpu::GainSTMMode {
    fn from(value: GainStmMode) -> Self {
        match value {
            GainStmMode::PhaseIntensityFull => Self::PhaseIntensityFull,
            GainStmMode::PhaseFull => Self::PhaseFull,
            GainStmMode::PhaseHalf => Self::PhaseHalf,
        }
    }
}

impl From<autd3_driver::firmware::cpu::GainSTMMode> for GainStmMode {
    fn from(value: autd3_driver::firmware::cpu::GainSTMMode) -> Self {
        match value {
            autd3_driver::firmware::cpu::GainSTMMode::PhaseIntensityFull => {
                Self::PhaseIntensityFull
            }
            autd3_driver::firmware::cpu::GainSTMMode::PhaseFull => Self::PhaseFull,
            autd3_driver::firmware::cpu::GainSTMMode::PhaseHalf => Self::PhaseHalf,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gain_stm_mode() {
        {
            let v = autd3_driver::firmware::cpu::GainSTMMode::PhaseIntensityFull;
            let msg: GainStmMode = v.into();
            let v2: autd3_driver::firmware::cpu::GainSTMMode = msg.into();
            assert_eq!(v, v2);
        }

        {
            let v = autd3_driver::firmware::cpu::GainSTMMode::PhaseFull;
            let msg: GainStmMode = v.into();
            let v2: autd3_driver::firmware::cpu::GainSTMMode = msg.into();
            assert_eq!(v, v2);
        }

        {
            let v = autd3_driver::firmware::cpu::GainSTMMode::PhaseHalf;
            let msg: GainStmMode = v.into();
            let v2: autd3_driver::firmware::cpu::GainSTMMode = msg.into();
            assert_eq!(v, v2);
        }
    }
}
