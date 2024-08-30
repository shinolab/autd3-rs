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

    fn apply_silencer_helper_interpolate(
        raw_seq: &[u8],
        update_rate: impl IntoIterator<Item = u16>,
        phase: bool,
        initial: u8,
    ) -> Vec<u8> {
        let mut current: i32 = (initial as i32) << 8;
        raw_seq
            .into_iter()
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

    fn apply_silencer_helper(
        &self,
        raw: &[u8],
        phase: bool,
        sampling_div: u16,
        initial: u8,
    ) -> Vec<u8> {
        let raw_seq = raw
            .iter()
            .flat_map(|i| std::iter::repeat(i).take(sampling_div as _))
            .cloned()
            .collect::<Vec<_>>();

        if self.silencer_fixed_update_rate_mode() {
            let update_rate = if phase {
                self.silencer_update_rate().1
            } else {
                self.silencer_update_rate().0
            };
            Self::apply_silencer_helper_interpolate(
                &raw_seq,
                std::iter::repeat(update_rate),
                phase,
                initial,
            )
        } else {
            let completion_steps = if phase {
                self.silencer_completion_steps().1
            } else {
                self.silencer_completion_steps().0
            };
            let mut current_target = initial;
            let mut diff_mem = 0;
            let mut step_rem_mem = 0;
            Self::apply_silencer_helper_interpolate(
                &raw_seq,
                raw_seq.iter().map(|&v| {
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
                    let step_quo = diff / completion_steps;
                    let step_rem = diff % completion_steps;
                    let update_rate = if rst {
                        step_rem_mem = step_rem;
                        step_quo
                    } else {
                        if step_rem_mem == 0 {
                            step_quo
                        } else {
                            step_rem_mem -= 1;
                            step_quo + 1
                        }
                    };
                    (update_rate as u16) << 8
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
        assert_eq!(
            expect,
            fpga.apply_silencer_helper(&input, phase, 1, initial)
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(vec![21, 33, 45, 57, 69, 81, 93, 105, 117, 128, 128], 10, false, 10, vec![128; 11])] //  case 1: intensity
    #[case(vec![21, 33, 45, 57, 69, 81, 93, 105, 117, 128, 128], 10, true, 10, vec![128; 11])] // case 1: phase
    #[case(vec![25, 51, 77, 103, 129, 155, 180, 205, 230, 255, 255], 10, false, 0, vec![255; 11])] // case 2: intensity
    #[case(vec![12, 25, 38, 51, 64, 77, 90, 103, 116, 128, 128], 10, true, 0, vec![128; 11])] // case 2: phase
    #[case(vec![25, 51, 77, 103, 129, 155, 180, 205, 230, 255, 255], 10, false, 0, vec![255; 11])] // case 3: intensity
    #[case(vec![254, 241, 228, 215, 202, 189, 176, 163, 151, 139, 139], 10, true, 10, vec![139; 11])] // case 3: phase
    #[case(vec![25, 51, 77, 103, 129, 155, 180, 205, 230, 255, 255], 10, false, 0, vec![255; 11])] // case 4: intensity
    #[case(vec![244, 231, 218, 205, 192, 179, 166, 153, 141, 129, 129], 10, true, 0, vec![129; 11])] // case 4: phase
    #[case(vec![25, 51, 77, 103, 129, 155, 180, 205, 230, 255, 255], 10, false, 0, vec![255; 11])] // case 5: intensity
    #[case(vec![249, 241, 233, 225, 217, 209, 201, 194, 187, 180, 180], 10, true, 0, vec![180; 11])] // case 5: phase
    #[case(vec![254, 253, 252, 251, 250, 249, 248, 247, 246, 245, 245], 10, false, 255, vec![245; 11])] // case 6: intensity
    #[case(vec![175, 169, 163, 158, 153, 148, 143, 138, 133, 128, 128], 10, true, 180, vec![128; 11])] // case 6: phase
    #[case(vec![254, 253, 252, 251, 250, 249, 248, 247, 246, 245, 245], 10, false, 255, vec![245; 11])] // case 7: intensity
    #[case(vec![248, 240, 232, 224, 216, 208, 201, 194, 187, 180, 180], 10, true, 255, vec![180; 11])] // case 7: phase
    #[case(vec![254, 253, 252, 251, 250, 249, 248, 247, 246, 245, 245], 10, false, 255, vec![245; 11])] // case 8: intensity
    #[case(vec![11, 24, 37, 50, 63, 76, 89, 102, 114, 126, 126], 10, true, 255, vec![126; 11])] // case 8: phase
    #[case(vec![254, 253, 252, 251, 250, 249, 248, 247, 246, 245, 245], 10, false, 255, vec![245; 11])] // case 9: intensity
    #[case(vec![243, 230, 217, 204, 191, 178, 165, 152, 139, 127, 127], 10, true, 255, vec![127; 11])] // case 9: phase
    #[case(vec![254, 253, 252, 251, 250, 249, 248, 247, 246, 245, 245], 10, false, 255, vec![245; 11])] // case 10: intensity
    #[case(vec![0, 2, 3, 4, 5, 6, 7, 8, 9, 10, 10], 10, true, 255, vec![10; 11])] // case 10: phase
    #[case(vec![254, 253, 252, 251, 250, 249, 248, 247, 246, 245, 245], 10, false, 255, vec![245; 11])] // case 11: intensity
    #[case(vec![187, 195, 203, 211, 219, 227, 235, 242, 249, 0, 0], 10, true, 180, vec![0; 11])] // case 11: phase
    #[case(vec![0, 1, 2, 3, 4, 5, 5, 5, 5, 5, 5], 10, false, 0, vec![5; 11])] // case 12: intensity
    #[case(vec![0, 1, 2, 3, 4, 5, 5, 5, 5, 5, 5], 10, true, 0, vec![5; 11])] // case 12: phase
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
        assert_eq!(
            expect,
            fpga.apply_silencer_helper(&input, phase, 1, initial)
        );
    }

}
