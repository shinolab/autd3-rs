use crate::{
    datagram::*,
    defined::DEFAULT_TIMEOUT,
    firmware::fpga::{
        SILENCER_STEPS_INTENSITY_DEFAULT, SILENCER_STEPS_PHASE_DEFAULT, SILENCER_VALUE_MAX,
        SILENCER_VALUE_MIN,
    },
};

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
    ) -> Result<Silencer<FixedUpdateRate>, AUTDInternalError> {
        if !(SILENCER_VALUE_MIN..=SILENCER_VALUE_MAX).contains(&update_rate_intensity) {
            return Err(AUTDInternalError::SilencerUpdateRateOutOfRange(
                update_rate_intensity,
            ));
        }
        if !(SILENCER_VALUE_MIN..=SILENCER_VALUE_MAX).contains(&update_rate_phase) {
            return Err(AUTDInternalError::SilencerUpdateRateOutOfRange(
                update_rate_phase,
            ));
        }
        Ok(Silencer {
            internal: FixedUpdateRate {
                update_rate_intensity,
                update_rate_phase,
            },
        })
    }

    pub fn fixed_completion_steps(
        steps_intensity: u16,
        steps_phase: u16,
    ) -> Result<Silencer<FixedCompletionSteps>, AUTDInternalError> {
        if !(SILENCER_VALUE_MIN..=SILENCER_VALUE_MAX).contains(&steps_intensity) {
            return Err(AUTDInternalError::SilencerCompletionStepsOutOfRange(
                steps_intensity,
            ));
        }
        if !(SILENCER_VALUE_MIN..=SILENCER_VALUE_MAX).contains(&steps_phase) {
            return Err(AUTDInternalError::SilencerCompletionStepsOutOfRange(
                steps_phase,
            ));
        }
        Ok(Silencer {
            internal: FixedCompletionSteps {
                steps_intensity,
                steps_phase,
                strict_mode: true,
            },
        })
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

impl<'a> Datagram<'a> for Silencer<FixedUpdateRate> {
    type O1 = crate::firmware::operation::ConfigSilencerFixedUpdateRateOp;
    type O2 = crate::firmware::operation::NullOp;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation(
        &'a self,
        _: &'a Geometry,
    ) -> Result<impl Fn(&'a Device) -> (Self::O1, Self::O2), AUTDInternalError> {
        Ok(move |_| {
            (
                Self::O1::new(
                    self.internal.update_rate_intensity,
                    self.internal.update_rate_phase,
                ),
                Self::O2::default(),
            )
        })
    }
}

impl<'a> Datagram<'a> for Silencer<FixedCompletionSteps> {
    type O1 = crate::firmware::operation::ConfigSilencerFixedCompletionStepsOp;
    type O2 = crate::firmware::operation::NullOp;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation(
        &'a self,
        _: &'a Geometry,
    ) -> Result<impl Fn(&'a Device) -> (Self::O1, Self::O2), AUTDInternalError> {
        Ok(move |_| {
            (
                Self::O1::new(
                    self.internal.steps_intensity,
                    self.internal.steps_phase,
                    self.internal.strict_mode,
                ),
                Self::O2::default(),
            )
        })
    }
}
