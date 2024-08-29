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

        let update_rate = if self.silencer_fixed_update_rate_mode() {
            let update_rate = if phase {
                self.silencer_update_rate().1
            } else {
                self.silencer_update_rate().0
            };
            vec![update_rate; raw_seq.len()]
        } else {
            let completion_steps = if phase {
                self.silencer_completion_steps().1
            } else {
                self.silencer_completion_steps().0
            };
            let mut current_target = initial;
            let mut diff_mem = 0;
            let mut step_rem_mem = 0;
            raw_seq
                .iter()
                .map(|&v| {
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
                })
                .collect::<Vec<_>>()
        };

        let mut current: i32 = 0;
        raw_seq
            .into_iter()
            .zip(update_rate)
            .map(|(v, u)| {
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
                        current += update_rate;
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[test]
    #[case([vec![0; 255], vec![1]].concat(), 1, false, vec![1; 256])]
    #[case([vec![0; 255], vec![1]].concat(), 1, true, vec![1; 256])]
    #[case([(1..=255).collect::<Vec<_>>(), vec![255]].concat(), 256, false, vec![255; 256])]
    #[case(vec![255; 256], 256, true, vec![255; 256])]
    #[cfg_attr(miri, ignore)]
    fn apply_silencer_fixed_update_rate(
        #[case] expect: Vec<u8>,
        #[case] value: u16,
        #[case] phase: bool,
        #[case] input: Vec<u8>,
    ) {
        let fpga = FPGAEmulator::new(249);
        fpga.mem.controller_bram_mut()[ADDR_SILENCER_FLAG] = SILENCER_FLAG_FIXED_UPDATE_RATE_MODE;
        if phase {
            fpga.mem.controller_bram_mut()[ADDR_SILENCER_UPDATE_RATE_PHASE] = value;
        } else {
            fpga.mem.controller_bram_mut()[ADDR_SILENCER_UPDATE_RATE_INTENSITY] = value;
        }
        assert_eq!(expect, fpga.apply_silencer_helper(&input, phase, 1, 0));
    }
}
