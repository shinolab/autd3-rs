/*
 * File: silencer.rs
 * Project: datagram
 * Created Date: 01/09/2023
 * Author: Shun Suzuki
 * -----
 * Last Modified: 28/12/2023
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2023 Shun Suzuki. All rights reserved.
 *
 */

use std::time::Duration;

use crate::{
    datagram::*,
    error::AUTDInternalError,
    fpga::{
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

#[derive(Debug, Clone, Copy)]
pub struct FixedUpdateRate {
    update_rate_intensity: u16,
    update_rate_phase: u16,
}

/// Datagram for configure silencer
#[derive(Debug, Clone, Copy)]
pub struct ConfigureSilencer<T> {
    internal: T,
}

impl ConfigureSilencer<()> {
    /// constructor
    ///
    /// # Arguments
    /// * `update_rate_intensity` - The intensity update rate of silencer. The lower the value, the stronger the silencer effect.
    /// * `update_rate_phase` - The phase update rate of silencer. The lower the value, the stronger the silencer effect.
    pub fn fixed_update_rate(
        update_rate_intensity: u16,
        update_rate_phase: u16,
    ) -> Result<ConfigureSilencer<FixedUpdateRate>, AUTDInternalError> {
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
        Ok(ConfigureSilencer {
            internal: FixedUpdateRate {
                update_rate_intensity,
                update_rate_phase,
            },
        })
    }

    /// constructor
    ///
    /// # Arguments
    /// * `steps_intensity` - The intensity completion steps of silencer. The higher the value, the stronger the silencer effect.
    /// * `steps_phase` - The phase completion steps of silencer. The higher the value, the stronger the silencer effect.
    pub fn fixed_completion_steps(
        steps_intensity: u16,
        steps_phase: u16,
    ) -> Result<ConfigureSilencer<FixedCompletionSteps>, AUTDInternalError> {
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
        Ok(ConfigureSilencer {
            internal: FixedCompletionSteps {
                steps_intensity,
                steps_phase,
                strict_mode: true,
            },
        })
    }

    /// Disable silencer
    pub const fn disable() -> ConfigureSilencer<FixedCompletionSteps> {
        ConfigureSilencer {
            internal: FixedCompletionSteps {
                steps_intensity: 1,
                steps_phase: 1,
                strict_mode: true,
            },
        }
    }
}

impl Default for ConfigureSilencer<FixedCompletionSteps> {
    fn default() -> Self {
        ConfigureSilencer {
            internal: FixedCompletionSteps {
                steps_intensity: SILENCER_STEPS_INTENSITY_DEFAULT,
                steps_phase: SILENCER_STEPS_PHASE_DEFAULT,
                strict_mode: true,
            },
        }
    }
}

impl ConfigureSilencer<FixedCompletionSteps> {
    /// Set strict mode
    pub fn with_strict_mode(mut self, strict_mode: bool) -> Self {
        self.internal.strict_mode = strict_mode;
        self
    }

    pub fn completion_steps_intensity(&self) -> u16 {
        self.internal.steps_intensity
    }

    pub fn completion_steps_phase(&self) -> u16 {
        self.internal.steps_phase
    }

    pub fn strict_mode(&self) -> bool {
        self.internal.strict_mode
    }
}

impl ConfigureSilencer<FixedUpdateRate> {
    pub fn update_rate_intensity(&self) -> u16 {
        self.internal.update_rate_intensity
    }

    pub fn update_rate_phase(&self) -> u16 {
        self.internal.update_rate_phase
    }
}

impl Datagram for ConfigureSilencer<FixedUpdateRate> {
    type O1 = crate::operation::ConfigSilencerFixedUpdateRateOp;
    type O2 = crate::operation::NullOp;

    fn timeout(&self) -> Option<Duration> {
        Some(Duration::from_millis(200))
    }

    fn operation(self) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
        Ok((
            Self::O1::new(
                self.internal.update_rate_intensity,
                self.internal.update_rate_phase,
            ),
            Self::O2::default(),
        ))
    }
}

impl Datagram for ConfigureSilencer<FixedCompletionSteps> {
    type O1 = crate::operation::ConfigSilencerFixedCompletionStepsOp;
    type O2 = crate::operation::NullOp;

    fn timeout(&self) -> Option<Duration> {
        Some(Duration::from_millis(200))
    }

    fn operation(self) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
        Ok((
            Self::O1::new(
                self.internal.steps_intensity,
                self.internal.steps_phase,
                self.internal.strict_mode,
            ),
            Self::O2::default(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_silencer_update_rate() {
        let silencer = ConfigureSilencer::fixed_update_rate(10, 20).unwrap();
        assert_eq!(silencer.update_rate_intensity(), 10);
        assert_eq!(silencer.update_rate_phase(), 20);
        assert_eq!(
            silencer.update_rate_intensity(),
            silencer.clone().update_rate_intensity()
        );
        assert_eq!(
            silencer.update_rate_phase(),
            silencer.clone().update_rate_phase()
        );

        let silencer = ConfigureSilencer::fixed_update_rate(0, 1);
        assert_eq!(
            silencer.unwrap_err(),
            AUTDInternalError::SilencerUpdateRateOutOfRange(0)
        );

        let silencer = ConfigureSilencer::fixed_update_rate(1, 0);
        assert_eq!(
            silencer.unwrap_err(),
            AUTDInternalError::SilencerUpdateRateOutOfRange(0)
        );
    }

    #[test]
    fn test_silencer_completion_steps() {
        let silencer = ConfigureSilencer::fixed_completion_steps(10, 20).unwrap();
        assert_eq!(silencer.completion_steps_intensity(), 10);
        assert_eq!(silencer.completion_steps_phase(), 20);
        assert_eq!(
            silencer.completion_steps_intensity(),
            silencer.clone().completion_steps_intensity()
        );
        assert_eq!(
            silencer.completion_steps_phase(),
            silencer.clone().completion_steps_phase()
        );

        let silencer = ConfigureSilencer::fixed_completion_steps(0, 1);
        assert_eq!(
            silencer.unwrap_err(),
            AUTDInternalError::SilencerCompletionStepsOutOfRange(0)
        );

        let silencer = ConfigureSilencer::fixed_completion_steps(1, 0);
        assert_eq!(
            silencer.unwrap_err(),
            AUTDInternalError::SilencerCompletionStepsOutOfRange(0)
        );
    }

    #[test]
    fn test_fixed_update_rate_debug() {
        let silencer = FixedUpdateRate {
            update_rate_intensity: 10,
            update_rate_phase: 20,
        };
        assert_eq!(
            format!("{:?}", silencer),
            "FixedUpdateRate { update_rate_intensity: 10, update_rate_phase: 20 }"
        );
    }

    #[test]
    fn test_fixed_completion_steps_debug() {
        let silencer = FixedCompletionSteps {
            steps_intensity: 10,
            steps_phase: 20,
            strict_mode: true,
        };
        assert_eq!(
            format!("{:?}", silencer),
            "FixedCompletionSteps { steps_intensity: 10, steps_phase: 20, strict_mode: true }"
        );
    }

    #[test]
    fn test_silencer_debug() {
        let silencer = ConfigureSilencer::fixed_update_rate(10, 20).unwrap();
        assert_eq!(
            format!("{:?}", silencer),
            "ConfigureSilencer { internal: FixedUpdateRate { update_rate_intensity: 10, update_rate_phase: 20 } }"
        );
    }

    #[test]
    fn test_silencer_disable() {
        let silencer = ConfigureSilencer::disable();
        assert_eq!(silencer.completion_steps_intensity(), 1);
        assert_eq!(silencer.completion_steps_phase(), 1);
        assert!(silencer.strict_mode());
    }

    #[test]
    fn test_silencer_default() {
        let silencer = ConfigureSilencer::default();
        assert_eq!(
            silencer.completion_steps_intensity(),
            SILENCER_STEPS_INTENSITY_DEFAULT
        );
        assert_eq!(
            silencer.completion_steps_phase(),
            SILENCER_STEPS_PHASE_DEFAULT
        );
        assert!(silencer.strict_mode());
    }

    #[test]
    fn test_silencer_timeout() {
        let silencer = ConfigureSilencer::fixed_update_rate(1, 2).unwrap();
        let timeout = silencer.timeout();
        assert!(timeout.is_some());
        assert!(timeout.unwrap() > Duration::ZERO);

        let silencer = ConfigureSilencer::fixed_completion_steps(1, 2).unwrap();
        let timeout = silencer.timeout();
        assert!(timeout.is_some());
        assert!(timeout.unwrap() > Duration::ZERO);
    }

    #[test]
    fn test_silencer_operation() {
        let silencer = ConfigureSilencer::fixed_update_rate(1, 2).unwrap();
        let r = silencer.operation();
        assert!(r.is_ok());
        let _: (
            crate::operation::ConfigSilencerFixedUpdateRateOp,
            crate::operation::NullOp,
        ) = r.unwrap();

        let silencer = ConfigureSilencer::fixed_completion_steps(1, 2).unwrap();
        let r = silencer.operation();
        assert!(r.is_ok());
        let _: (
            crate::operation::ConfigSilencerFixedCompletionStepsOp,
            crate::operation::NullOp,
        ) = r.unwrap();
    }
}
