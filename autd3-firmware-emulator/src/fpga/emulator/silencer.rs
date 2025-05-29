use std::num::NonZeroU16;

use autd3_driver::{
    datagram::{FixedCompletionSteps, FixedUpdateRate},
    firmware::fpga::{EmitIntensity, Phase},
};

use super::{super::params::*, FPGAEmulator};

#[derive(Debug, Clone, Copy)]
pub struct SilencerEmulator<T> {
    current: i32,
    fixed_update_rate_mode: bool,
    value: u16,
    current_target: u8,
    diff_mem: u8,
    step_rem_mem: u16,
    _phantom: std::marker::PhantomData<T>,
}

impl SilencerEmulator<Phase> {
    #[allow(clippy::collapsible_else_if)]
    #[must_use]
    fn update_rate(&mut self, input: u8) -> u16 {
        if self.fixed_update_rate_mode {
            self.value
        } else {
            let diff = if input < self.current_target {
                self.current_target - input
            } else {
                input - self.current_target
            };
            self.current_target = input;
            let diff = if diff >= 128 {
                (256 - diff as u16) as u8
            } else {
                diff
            };
            let (diff, rst) = if diff == 0 {
                (self.diff_mem, false)
            } else {
                self.diff_mem = diff;
                (diff, true)
            };
            let step_quo = ((diff as u16) << 8) / self.value;
            let step_rem = ((diff as u16) << 8) % self.value;
            if rst {
                self.step_rem_mem = step_rem;
                step_quo
            } else {
                if self.step_rem_mem == 0 {
                    step_quo
                } else {
                    self.step_rem_mem -= 1;
                    step_quo + 1
                }
            }
        }
    }

    #[allow(clippy::collapsible_else_if)]
    #[must_use]
    pub fn apply(&mut self, input: u8) -> u8 {
        let update_rate = self.update_rate(input) as i32;
        let step = ((input as i32) << 8) - self.current;
        let step = if step < 0 {
            if -32768 <= step { step } else { step + 65536 }
        } else {
            if step <= 32768 { step } else { step - 65536 }
        };
        if step < 0 {
            if -update_rate <= step {
                self.current += step;
            } else {
                self.current -= update_rate;
            }
        } else {
            if step <= update_rate {
                self.current += step;
            } else {
                self.current += update_rate;
            }
        }
        (self.current >> 8) as u8
    }
}

impl SilencerEmulator<EmitIntensity> {
    #[allow(clippy::collapsible_else_if)]
    #[must_use]
    fn update_rate(&mut self, input: u8) -> u16 {
        if self.fixed_update_rate_mode {
            self.value
        } else {
            let diff = if input < self.current_target {
                self.current_target - input
            } else {
                input - self.current_target
            };
            self.current_target = input;
            let (diff, rst) = if diff == 0 {
                (self.diff_mem, false)
            } else {
                self.diff_mem = diff;
                (diff, true)
            };
            let step_quo = ((diff as u16) << 8) / self.value;
            let step_rem = ((diff as u16) << 8) % self.value;
            if rst {
                self.step_rem_mem = step_rem;
                step_quo
            } else {
                if self.step_rem_mem == 0 {
                    step_quo
                } else {
                    self.step_rem_mem -= 1;
                    step_quo + 1
                }
            }
        }
    }

    #[allow(clippy::collapsible_else_if)]
    #[must_use]
    pub fn apply(&mut self, input: u8) -> u8 {
        let update_rate = self.update_rate(input) as i32;
        let step = ((input as i32) << 8) - self.current;
        if step < 0 {
            if -update_rate <= step {
                self.current += step;
            } else {
                self.current -= update_rate;
            }
        } else {
            if step <= update_rate {
                self.current += step;
            } else {
                self.current += update_rate;
            }
        }
        (self.current >> 8) as u8
    }
}

impl FPGAEmulator {
    #[must_use]
    pub fn silencer_update_rate(&self) -> FixedUpdateRate {
        unsafe {
            FixedUpdateRate {
                intensity: NonZeroU16::new_unchecked(
                    self.mem
                        .controller_bram
                        .read(ADDR_SILENCER_UPDATE_RATE_INTENSITY),
                ),
                phase: NonZeroU16::new_unchecked(
                    self.mem
                        .controller_bram
                        .read(ADDR_SILENCER_UPDATE_RATE_PHASE),
                ),
            }
        }
    }

    #[must_use]
    pub fn silencer_completion_steps(&self) -> FixedCompletionSteps {
        FixedCompletionSteps {
            intensity: NonZeroU16::new(
                self.mem
                    .controller_bram
                    .read(ADDR_SILENCER_COMPLETION_STEPS_INTENSITY),
            )
            .unwrap(),
            phase: NonZeroU16::new(
                self.mem
                    .controller_bram
                    .read(ADDR_SILENCER_COMPLETION_STEPS_PHASE),
            )
            .unwrap(),
            strict_mode: true,
        }
    }

    #[must_use]
    pub fn silencer_fixed_update_rate_mode(&self) -> bool {
        (self.mem.controller_bram.read(ADDR_SILENCER_FLAG) & SILENCER_FLAG_FIXED_UPDATE_RATE_MODE)
            == SILENCER_FLAG_FIXED_UPDATE_RATE_MODE
    }

    #[must_use]
    pub fn silencer_fixed_completion_steps_mode(&self) -> bool {
        !self.silencer_fixed_update_rate_mode()
    }

    #[must_use]
    pub fn silencer_emulator_phase(&self, initial: u8) -> SilencerEmulator<Phase> {
        SilencerEmulator {
            current: (initial as i32) << 8,
            fixed_update_rate_mode: self.silencer_fixed_update_rate_mode(),
            value: if self.silencer_fixed_update_rate_mode() {
                self.silencer_update_rate().phase.get()
            } else {
                self.silencer_completion_steps().phase.get()
            },
            current_target: initial,
            diff_mem: 0,
            step_rem_mem: 0,
            _phantom: std::marker::PhantomData,
        }
    }

    #[must_use]
    pub fn silencer_emulator_phase_continue_with(
        &self,
        prev: SilencerEmulator<Phase>,
    ) -> SilencerEmulator<Phase> {
        let SilencerEmulator {
            current,
            fixed_update_rate_mode: _fixed_update_rate_mode,
            value: _value,
            current_target,
            diff_mem,
            step_rem_mem,
            _phantom,
        } = prev;

        SilencerEmulator {
            current,
            fixed_update_rate_mode: self.silencer_fixed_update_rate_mode(),
            value: if self.silencer_fixed_update_rate_mode() {
                self.silencer_update_rate().phase.get()
            } else {
                self.silencer_completion_steps().phase.get()
            },
            current_target,
            diff_mem,
            step_rem_mem,
            _phantom: std::marker::PhantomData,
        }
    }

    #[must_use]
    pub fn silencer_emulator_intensity(&self, initial: u8) -> SilencerEmulator<EmitIntensity> {
        SilencerEmulator {
            current: (initial as i32) << 8,
            fixed_update_rate_mode: self.silencer_fixed_update_rate_mode(),
            value: if self.silencer_fixed_update_rate_mode() {
                self.silencer_update_rate().intensity.get()
            } else {
                self.silencer_completion_steps().intensity.get()
            },
            current_target: initial,
            diff_mem: 0,
            step_rem_mem: 0,
            _phantom: std::marker::PhantomData,
        }
    }

    #[must_use]
    pub fn silencer_emulator_intensity_continue_with(
        &self,
        prev: SilencerEmulator<EmitIntensity>,
    ) -> SilencerEmulator<EmitIntensity> {
        let SilencerEmulator {
            current,
            fixed_update_rate_mode: _fixed_update_rate_mode,
            value: _value,
            current_target,
            diff_mem,
            step_rem_mem,
            _phantom,
        } = prev;

        SilencerEmulator {
            current,
            fixed_update_rate_mode: self.silencer_fixed_update_rate_mode(),
            value: if self.silencer_fixed_update_rate_mode() {
                self.silencer_update_rate().intensity.get()
            } else {
                self.silencer_completion_steps().intensity.get()
            },
            current_target,
            diff_mem,
            step_rem_mem,
            _phantom: std::marker::PhantomData,
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
    fn apply_silencer_fixed_update_rate(
        #[case] expect: Vec<u8>,
        #[case] value: u16,
        #[case] phase: bool,
        #[case] initial: u8,
        #[case] input: Vec<u8>,
    ) {
        let fpga = FPGAEmulator::new(249);
        fpga.mem
            .controller_bram
            .write(ADDR_SILENCER_FLAG, SILENCER_FLAG_FIXED_UPDATE_RATE_MODE);
        fpga.mem
            .controller_bram
            .write(ADDR_SILENCER_UPDATE_RATE_PHASE, value);
        fpga.mem
            .controller_bram
            .write(ADDR_SILENCER_UPDATE_RATE_INTENSITY, value);
        if phase {
            let mut silencer = fpga.silencer_emulator_phase(initial);
            assert_eq!(
                expect,
                input
                    .into_iter()
                    .map(|i| silencer.apply(i))
                    .collect::<Vec<_>>()
            );
        } else {
            let mut silencer = fpga.silencer_emulator_intensity(initial);
            assert_eq!(
                expect,
                input
                    .into_iter()
                    .map(|i| silencer.apply(i))
                    .collect::<Vec<_>>()
            );
        }
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
    fn apply_silencer_fixed_completion_steps(
        #[case] expect: Vec<u8>,
        #[case] value: u8,
        #[case] phase: bool,
        #[case] initial: u8,
        #[case] input: Vec<u8>,
    ) {
        let fpga = FPGAEmulator::new(249);
        fpga.mem
            .controller_bram
            .write(ADDR_SILENCER_COMPLETION_STEPS_PHASE, value as _);
        fpga.mem
            .controller_bram
            .write(ADDR_SILENCER_COMPLETION_STEPS_INTENSITY, value as _);
        if phase {
            let mut silencer = fpga.silencer_emulator_phase(initial);
            assert_eq!(
                expect,
                input
                    .into_iter()
                    .map(|i| silencer.apply(i))
                    .collect::<Vec<_>>()
            );
        } else {
            let mut silencer = fpga.silencer_emulator_intensity(initial);
            assert_eq!(
                expect,
                input
                    .into_iter()
                    .map(|i| silencer.apply(i))
                    .collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn silencer_emulator_phase_continue_with() {
        let fpga = FPGAEmulator::new(249);
        fpga.mem
            .controller_bram
            .write(ADDR_SILENCER_COMPLETION_STEPS_PHASE, 0x01);
        fpga.mem
            .controller_bram
            .write(ADDR_SILENCER_COMPLETION_STEPS_INTENSITY, 0x01);

        let mut silencer = fpga.silencer_emulator_phase(0);
        _ = silencer.apply(0xFF);

        let SilencerEmulator {
            current,
            fixed_update_rate_mode: _,
            value: _,
            current_target,
            diff_mem,
            step_rem_mem,
            _phantom,
        } = silencer;

        fpga.mem
            .controller_bram
            .write(ADDR_SILENCER_COMPLETION_STEPS_PHASE, 0x02);
        fpga.mem
            .controller_bram
            .write(ADDR_SILENCER_COMPLETION_STEPS_INTENSITY, 0x02);
        let silencer = fpga.silencer_emulator_phase_continue_with(silencer);

        assert_eq!(current, silencer.current);
        assert!(!silencer.fixed_update_rate_mode);
        assert_eq!(0x02, silencer.value);
        assert_eq!(current_target, silencer.current_target);
        assert_eq!(diff_mem, silencer.diff_mem);
        assert_eq!(step_rem_mem, silencer.step_rem_mem);

        let SilencerEmulator {
            current,
            fixed_update_rate_mode: _,
            value: _,
            current_target,
            diff_mem,
            step_rem_mem,
            _phantom,
        } = silencer;

        fpga.mem
            .controller_bram
            .write(ADDR_SILENCER_UPDATE_RATE_PHASE, 0x03);
        fpga.mem
            .controller_bram
            .write(ADDR_SILENCER_UPDATE_RATE_INTENSITY, 0x03);
        fpga.mem.controller_bram.write(
            ADDR_SILENCER_FLAG,
            1 << SILENCER_FLAG_BIT_FIXED_UPDATE_RATE_MODE,
        );

        let silencer = fpga.silencer_emulator_phase_continue_with(silencer);

        assert_eq!(current, silencer.current);
        assert!(silencer.fixed_update_rate_mode);
        assert_eq!(0x03, silencer.value);
        assert_eq!(current_target, silencer.current_target);
        assert_eq!(diff_mem, silencer.diff_mem);
        assert_eq!(step_rem_mem, silencer.step_rem_mem);
    }

    #[test]
    fn silencer_emulator_intensity_continue_with() {
        let fpga = FPGAEmulator::new(249);
        fpga.mem
            .controller_bram
            .write(ADDR_SILENCER_COMPLETION_STEPS_PHASE, 0x01);
        fpga.mem
            .controller_bram
            .write(ADDR_SILENCER_COMPLETION_STEPS_INTENSITY, 0x01);

        let mut silencer = fpga.silencer_emulator_intensity(0);
        _ = silencer.apply(0xFF);

        let SilencerEmulator {
            current,
            fixed_update_rate_mode: _,
            value: _,
            current_target,
            diff_mem,
            step_rem_mem,
            _phantom,
        } = silencer;

        fpga.mem
            .controller_bram
            .write(ADDR_SILENCER_COMPLETION_STEPS_PHASE, 0x02);
        fpga.mem
            .controller_bram
            .write(ADDR_SILENCER_COMPLETION_STEPS_INTENSITY, 0x02);
        let silencer = fpga.silencer_emulator_intensity_continue_with(silencer);

        assert_eq!(current, silencer.current);
        assert!(!silencer.fixed_update_rate_mode);
        assert_eq!(0x02, silencer.value);
        assert_eq!(current_target, silencer.current_target);
        assert_eq!(diff_mem, silencer.diff_mem);
        assert_eq!(step_rem_mem, silencer.step_rem_mem);

        let SilencerEmulator {
            current,
            fixed_update_rate_mode: _,
            value: _,
            current_target,
            diff_mem,
            step_rem_mem,
            _phantom,
        } = silencer;

        fpga.mem
            .controller_bram
            .write(ADDR_SILENCER_UPDATE_RATE_PHASE, 0x03);
        fpga.mem
            .controller_bram
            .write(ADDR_SILENCER_UPDATE_RATE_INTENSITY, 0x03);
        fpga.mem.controller_bram.write(
            ADDR_SILENCER_FLAG,
            1 << SILENCER_FLAG_BIT_FIXED_UPDATE_RATE_MODE,
        );

        let silencer = fpga.silencer_emulator_intensity_continue_with(silencer);

        assert_eq!(current, silencer.current);
        assert!(silencer.fixed_update_rate_mode);
        assert_eq!(0x03, silencer.value);
        assert_eq!(current_target, silencer.current_target);
        assert_eq!(diff_mem, silencer.diff_mem);
        assert_eq!(step_rem_mem, silencer.step_rem_mem);
    }
}
