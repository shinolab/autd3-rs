use autd3_driver::firmware::fpga::SilencerTarget;

use super::{super::params::*, FPGAEmulator};

impl FPGAEmulator {
    pub fn silencer_update_rate(&self) -> (u16, u16) {
        (
            self.mem.controller_bram()[ADDR_SILENCER_UPDATE_RATE_INTENSITY],
            self.mem.controller_bram()[ADDR_SILENCER_UPDATE_RATE_PHASE],
        )
    }

    pub fn silencer_completion_steps(&self) -> (u16, u16) {
        (
            self.mem.controller_bram()[ADDR_SILENCER_COMPLETION_STEPS_INTENSITY],
            self.mem.controller_bram()[ADDR_SILENCER_COMPLETION_STEPS_PHASE],
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

    #[allow(clippy::collapsible_else_if)]
    fn apply_silencer_interpolate(
        raw_seq: &[u8],
        update_rate: impl IntoIterator<Item = u16>,
        phase: bool,
        initial: u8,
    ) -> Vec<u8> {
        let mut current: i32 = (initial as i32) << 8;
        raw_seq
            .iter()
            .zip(update_rate)
            .map(|(&v, u)| {
                let update_rate = u as i32;
                let step = ((v as i32) << 8) - current;
                let step = if phase {
                    if step < 0 {
                        if -32768 <= step {
                            step
                        } else {
                            step + 65536
                        }
                    } else {
                        if step <= 32768 {
                            step
                        } else {
                            step - 65536
                        }
                    }
                } else {
                    step
                };
                if step < 0 {
                    if -update_rate <= step {
                        current += step;
                    } else {
                        current -= update_rate;
                    }
                } else {
                    if step <= update_rate {
                        current += step;
                    } else {
                        current += update_rate;
                    }
                }
                (current >> 8) as u8
            })
            .collect()
    }

    #[allow(clippy::collapsible_else_if)]
    pub fn apply_silencer(&self, initial: u8, raw: &[u8], phase: bool) -> Vec<u8> {
        if self.silencer_fixed_update_rate_mode() {
            let update_rate = if phase {
                self.silencer_update_rate().1
            } else {
                self.silencer_update_rate().0
            };
            Self::apply_silencer_interpolate(raw, std::iter::repeat(update_rate), phase, initial)
        } else {
            let completion_steps = if phase {
                self.silencer_completion_steps().1
            } else {
                self.silencer_completion_steps().0
            };
            let mut current_target = initial;
            let mut diff_mem = 0;
            let mut step_rem_mem = 0;
            Self::apply_silencer_interpolate(
                raw,
                raw.iter().map(|&v| {
                    let diff = if v < current_target {
                        current_target - v
                    } else {
                        v - current_target
                    };
                    current_target = v;
                    let diff = if phase && diff >= 128 {
                        (256 - diff as u16) as u8
                    } else {
                        diff
                    };
                    let (diff, rst) = if diff == 0 {
                        (diff_mem, false)
                    } else {
                        diff_mem = diff;
                        (diff, true)
                    };
                    let step_quo = ((diff as u16) << 8) / completion_steps;
                    let step_rem = ((diff as u16) << 8) % completion_steps;
                    if rst {
                        step_rem_mem = step_rem;
                        step_quo
                    } else {
                        if step_rem_mem == 0 {
                            step_quo
                        } else {
                            step_rem_mem -= 1;
                            step_quo + 1
                        }
                    }
                }),
                phase,
                initial,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[test]
    #[case([vec![0; 255], vec![1]].concat(), 1, false, 0, vec![1; 256])]
    #[case([vec![0; 255], vec![1]].concat(), 1, true, 0, vec![1; 256])]
    #[case([vec![1; 256], vec![0]].concat(), 1, false, 2, vec![0; 257])]
    #[case([vec![1; 256], vec![0]].concat(), 1, true, 2, vec![0; 257])]
    #[case([(1..=255).collect::<Vec<_>>(), vec![255]].concat(), 256, false, 0, vec![255; 256])]
    #[case(vec![255; 256], 256, true, 0, vec![255; 256])]
    #[case([(0..=254).rev().collect::<Vec<_>>(), vec![0]].concat(), 256, false, 255, vec![0; 256])]
    #[case(vec![0; 256], 256, true, 255, vec![0; 256])]
    #[cfg_attr(miri, ignore)]
    fn apply_silencer_fixed_update_rate(
        #[case] expect: Vec<u8>,
        #[case] value: u16,
        #[case] phase: bool,
        #[case] initial: u8,
        #[case] input: Vec<u8>,
    ) {
        let fpga = FPGAEmulator::new(249);
        fpga.mem.controller_bram_mut()[ADDR_SILENCER_FLAG] = SILENCER_FLAG_FIXED_UPDATE_RATE_MODE;
        if phase {
            fpga.mem.controller_bram_mut()[ADDR_SILENCER_UPDATE_RATE_PHASE] = value;
        } else {
            fpga.mem.controller_bram_mut()[ADDR_SILENCER_UPDATE_RATE_INTENSITY] = value;
        }
        assert_eq!(expect, fpga.apply_silencer(initial, &input, phase));
    }

    #[rstest::rstest]
    #[test]
    #[case::intensity_1(vec![21, 33, 45, 57, 69, 80, 92, 104, 116, 128, 128], 10, false, 10, vec![128; 11])]
    #[case::phase_1(vec![21, 33, 45, 57, 69, 80, 92, 104, 116, 128, 128], 10, true, 10, vec![128; 11])]
    #[case::intensity_2(vec![25, 51, 76, 102, 127, 153, 178, 204, 229, 255, 255], 10, false, 0, vec![255; 11])]
    #[case::phase_2(vec![12, 25, 38, 51, 64, 76, 89, 102, 115, 128, 128], 10, true, 0, vec![128; 11])]
    #[case::intensity_3(vec![25, 51, 76, 102, 127, 153, 178, 204, 229, 255, 255], 10, false, 0, vec![255; 11])]
    #[case::phase_3(vec![253, 240, 227, 215, 202, 189, 177, 164, 151, 139, 139], 10, true, 10, vec![139; 11])]
    #[case::intensity_4(vec![25, 51, 76, 102, 127, 153, 178, 204, 229, 255, 255], 10, false, 0, vec![255; 11])]
    #[case::phase_4(vec![243, 230, 217, 205, 192, 179, 167, 154, 141, 129, 129], 10, true, 0, vec![129; 11])]
    #[case::intensity_5(vec![25, 51, 76, 102, 127, 153, 178, 204, 229, 255, 255], 10, false, 0, vec![255; 11])]
    #[case::phase_5(vec![248, 240, 233, 225, 217, 210, 202, 195, 187, 180, 180], 10, true, 0, vec![180; 11])]
    #[case::intensity_6(vec![254, 253, 252, 251, 250, 249, 248, 247, 246, 245, 245], 10, false, 255, vec![245; 11])]
    #[case::phase_6(vec![174, 169, 164, 159, 153, 148, 143, 138, 133, 128, 128], 10, true, 180, vec![128; 11])]
    #[case::intensity_7(vec![254, 253, 252, 251, 250, 249, 248, 247, 246, 245, 245], 10, false, 255, vec![245; 11])]
    #[case::phase_7(vec![247, 240, 232, 225, 217, 210, 202, 195, 187, 180, 180], 10, true, 255, vec![180; 11])]
    #[case::intensity_8(vec![254, 253, 252, 251, 250, 249, 248, 247, 246, 245, 245], 10, false, 255, vec![245; 11])]
    #[case::phase_8(vec![11, 24, 37, 49, 62, 75, 87, 100, 113, 126, 126], 10, true, 255, vec![126; 11])]
    #[case::intensity_9(vec![254, 253, 252, 251, 250, 249, 248, 247, 246, 245, 245], 10, false, 255, vec![245; 11])]
    #[case::phase_9(vec![242, 229, 216, 203, 191, 178, 165, 152, 139, 127, 127], 10, true, 255, vec![127; 11])]
    #[case::intensity_10(vec![254, 253, 252, 251, 250, 249, 248, 247, 246, 245, 245], 10, false, 255, vec![245; 11])]
    #[case::phase_10(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 10, 10], 10, true, 255, vec![10; 11])]
    #[case::intensity_11(vec![254, 253, 252, 251, 250, 249, 248, 247, 246, 245, 245], 10, false, 255, vec![245; 11])]
    #[case::phase_11(vec![187, 195, 202, 210, 218, 225, 233, 240, 248, 0, 0], 10, true, 180, vec![0; 11])]
    #[case::intensity_12(vec![0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5], 10, false, 0, vec![5; 11])]
    #[case::phase_12(vec![0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5], 10, true, 0, vec![5; 11])]
    #[cfg_attr(miri, ignore)]
    fn apply_silencer_fixed_completion_steps(
        #[case] expect: Vec<u8>,
        #[case] value: u8,
        #[case] phase: bool,
        #[case] initial: u8,
        #[case] input: Vec<u8>,
    ) {
        let fpga = FPGAEmulator::new(249);
        if phase {
            fpga.mem.controller_bram_mut()[ADDR_SILENCER_COMPLETION_STEPS_PHASE] = value as _;
        } else {
            fpga.mem.controller_bram_mut()[ADDR_SILENCER_COMPLETION_STEPS_INTENSITY] = value as _;
        }
        assert_eq!(expect, fpga.apply_silencer(initial, &input, phase));
    }
}
