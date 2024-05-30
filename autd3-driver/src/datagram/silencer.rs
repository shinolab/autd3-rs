use crate::firmware::{
    fpga::{SILENCER_STEPS_INTENSITY_DEFAULT, SILENCER_STEPS_PHASE_DEFAULT},
    operation::{SilencerFixedCompletionStepsOp, SilencerFixedUpdateRateOp},
};

use crate::datagram::*;

#[derive(Debug, Clone, Copy)]
pub struct FixedCompletionSteps {
    steps_intensity: u16,
    steps_phase: u16,
    strict_mode: bool,
}

impl std::ops::Mul<u16> for FixedCompletionSteps {
    type Output = Self;

    fn mul(self, rhs: u16) -> Self::Output {
        Self {
            steps_intensity: self.steps_intensity * rhs,
            steps_phase: self.steps_phase * rhs,
            strict_mode: self.strict_mode,
        }
    }
}

impl std::ops::Div<u16> for FixedCompletionSteps {
    type Output = Self;

    fn div(self, rhs: u16) -> Self::Output {
        Self {
            steps_intensity: self.steps_intensity / rhs,
            steps_phase: self.steps_phase / rhs,
            strict_mode: self.strict_mode,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FixedUpdateRate {
    update_rate_intensity: u16,
    update_rate_phase: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct Silencer<T> {
    internal: T,
}

pub type SilencerFixedCompletionSteps = Silencer<FixedCompletionSteps>;
pub type SilencerFixedUpdateRate = Silencer<FixedUpdateRate>;

impl Silencer<()> {
    pub fn fixed_update_rate(
        update_rate_intensity: u16,
        update_rate_phase: u16,
    ) -> Silencer<FixedUpdateRate> {
        Silencer {
            internal: FixedUpdateRate {
                update_rate_intensity,
                update_rate_phase,
            },
        }
    }

    pub fn fixed_completion_steps(
        steps_intensity: u16,
        steps_phase: u16,
    ) -> Silencer<FixedCompletionSteps> {
        Silencer {
            internal: FixedCompletionSteps {
                steps_intensity,
                steps_phase,
                strict_mode: true,
            },
        }
    }

    pub const fn disable() -> Silencer<FixedCompletionSteps> {
        Silencer {
            internal: FixedCompletionSteps {
                steps_intensity: 1,
                steps_phase: 1,
                strict_mode: true,
            },
        }
    }
}

impl<T> std::ops::Mul<u16> for Silencer<T>
where
    T: std::ops::Mul<u16, Output = T>,
{
    type Output = Silencer<T>;

    fn mul(self, rhs: u16) -> Self::Output {
        Silencer {
            internal: self.internal * rhs,
        }
    }
}

impl<T> std::ops::Div<u16> for Silencer<T>
where
    T: std::ops::Div<u16, Output = T>,
{
    type Output = Silencer<T>;

    fn div(self, rhs: u16) -> Self::Output {
        Silencer {
            internal: self.internal / rhs,
        }
    }
}

impl Default for Silencer<FixedCompletionSteps> {
    fn default() -> Self {
        Silencer {
            internal: FixedCompletionSteps {
                steps_intensity: SILENCER_STEPS_INTENSITY_DEFAULT,
                steps_phase: SILENCER_STEPS_PHASE_DEFAULT,
                strict_mode: true,
            },
        }
    }
}

impl Silencer<FixedCompletionSteps> {
    pub const fn with_strict_mode(mut self, strict_mode: bool) -> Self {
        self.internal.strict_mode = strict_mode;
        self
    }

    pub const fn completion_steps_intensity(&self) -> u16 {
        self.internal.steps_intensity
    }

    pub const fn completion_steps_phase(&self) -> u16 {
        self.internal.steps_phase
    }

    pub const fn strict_mode(&self) -> bool {
        self.internal.strict_mode
    }
}

impl Silencer<FixedUpdateRate> {
    pub const fn update_rate_intensity(&self) -> u16 {
        self.internal.update_rate_intensity
    }

    pub const fn update_rate_phase(&self) -> u16 {
        self.internal.update_rate_phase
    }
}

pub struct SilencerFixedUpdateRateOpGenerator {
    update_rate_intensity: u16,
    update_rate_phase: u16,
}

impl OperationGenerator for SilencerFixedUpdateRateOpGenerator {
    type O1 = SilencerFixedUpdateRateOp;
    type O2 = NullOp;

    fn generate(&self, _: &Device) -> (Self::O1, Self::O2) {
        (
            Self::O1::new(self.update_rate_intensity, self.update_rate_phase),
            Self::O2::default(),
        )
    }
}

impl Datagram for Silencer<FixedUpdateRate> {
    type O1 = SilencerFixedUpdateRateOp;
    type O2 = NullOp;
    type G = SilencerFixedUpdateRateOpGenerator;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation_generator(self, _: &Geometry) -> Result<Self::G, AUTDInternalError> {
        Ok(SilencerFixedUpdateRateOpGenerator {
            update_rate_intensity: self.internal.update_rate_intensity,
            update_rate_phase: self.internal.update_rate_phase,
        })
    }

    fn parallel_threshold(&self) -> Option<usize> {
        Some(usize::MAX)
    }
}

pub struct SilencerFixedCompletionStepsOpGenerator {
    update_rate_intensity: u16,
    update_rate_phase: u16,
    strict_mode: bool,
}

impl OperationGenerator for SilencerFixedCompletionStepsOpGenerator {
    type O1 = SilencerFixedCompletionStepsOp;
    type O2 = NullOp;

    fn generate(&self, _: &Device) -> (Self::O1, Self::O2) {
        (
            Self::O1::new(
                self.update_rate_intensity,
                self.update_rate_phase,
                self.strict_mode,
            ),
            Self::O2::default(),
        )
    }
}

impl Datagram for Silencer<FixedCompletionSteps> {
    type O1 = SilencerFixedCompletionStepsOp;
    type O2 = NullOp;
    type G = SilencerFixedCompletionStepsOpGenerator;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation_generator(self, _: &Geometry) -> Result<Self::G, AUTDInternalError> {
        Ok(SilencerFixedCompletionStepsOpGenerator {
            update_rate_intensity: self.internal.steps_intensity,
            update_rate_phase: self.internal.steps_phase,
            strict_mode: self.internal.strict_mode,
        })
    }

    fn parallel_threshold(&self) -> Option<usize> {
        Some(usize::MAX)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disable() {
        let s = Silencer::disable();
        assert_eq!(s.completion_steps_intensity(), 1);
        assert_eq!(s.completion_steps_phase(), 1);
        assert!(s.strict_mode());
    }

    #[test]
    fn fixed_update_rate() {
        let s = Silencer::fixed_update_rate(1, 2);
        assert_eq!(s.update_rate_intensity(), 1);
        assert_eq!(s.update_rate_phase(), 2);
    }

    #[test]
    fn fixed_completion_steps_mul() {
        let s = Silencer::fixed_completion_steps(1, 1);
        let s = s * 2;
        assert_eq!(s.completion_steps_intensity(), 2);
        assert_eq!(s.completion_steps_phase(), 2);
    }

    #[test]
    fn fixed_completion_steps_div() {
        let s = Silencer::fixed_completion_steps(2, 2);
        let s = s / 2;
        assert_eq!(s.completion_steps_intensity(), 1);
        assert_eq!(s.completion_steps_phase(), 1);
    }
}
