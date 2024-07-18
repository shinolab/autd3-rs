mod fixed_complition_steps;
mod fixed_complition_time;
mod fixed_update_rate;

use std::time::Duration;

pub use fixed_complition_steps::FixedCompletionSteps;
pub use fixed_complition_time::FixedCompletionTime;
pub use fixed_update_rate::FixedUpdateRate;

use derive_more::{Div, Mul};

use crate::firmware::operation::SilencerTarget;

#[derive(Debug, Clone, Copy, Mul, Div)]
pub struct Silencer<T> {
    internal: T,
}

pub type SilencerFixedCompletionSteps = Silencer<FixedCompletionSteps>;
pub type SilencerFixedCompletionTime = Silencer<FixedCompletionTime>;
pub type SilencerFixedUpdateRate = Silencer<FixedUpdateRate>;

impl Silencer<()> {
    pub const fn from_update_rate(
        update_rate_intensity: u16,
        update_rate_phase: u16,
    ) -> Silencer<FixedUpdateRate> {
        Silencer {
            internal: FixedUpdateRate {
                update_rate_intensity,
                update_rate_phase,
                target: SilencerTarget::Intensity,
            },
        }
    }

    pub const fn from_completion_steps(
        steps_intensity: u16,
        steps_phase: u16,
    ) -> Silencer<FixedCompletionSteps> {
        Silencer {
            internal: FixedCompletionSteps {
                steps_intensity,
                steps_phase,
                strict_mode: true,
                target: SilencerTarget::Intensity,
            },
        }
    }

    pub const fn from_completion_time(
        time_intensity: Duration,
        time_phase: Duration,
    ) -> Silencer<FixedCompletionTime> {
        Silencer {
            internal: FixedCompletionTime {
                time_intensity,
                time_phase,
                strict_mode: true,
                target: SilencerTarget::Intensity,
            },
        }
    }

    pub const fn disable() -> Silencer<FixedCompletionSteps> {
        Silencer {
            internal: FixedCompletionSteps {
                steps_intensity: 1,
                steps_phase: 1,
                strict_mode: true,
                target: SilencerTarget::Intensity,
            },
        }
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
    fn from_update_rate() {
        let s = Silencer::from_update_rate(1, 2);
        assert_eq!(1, s.update_rate_intensity());
        assert_eq!(2, s.update_rate_phase());
        assert_eq!(SilencerTarget::Intensity, s.target());
    }

    #[test]
    fn from_completion_steps_mul() {
        let s = Silencer::from_completion_steps(1, 1);
        let s = s * 2;
        assert_eq!(2, s.completion_steps_intensity());
        assert_eq!(2, s.completion_steps_phase());
        assert_eq!(SilencerTarget::Intensity, s.target());
    }

    #[test]
    fn from_completion_steps_div() {
        let s = Silencer::from_completion_steps(2, 2);
        let s = s / 2;
        assert_eq!(1, s.completion_steps_intensity());
        assert_eq!(1, s.completion_steps_phase());
        assert_eq!(SilencerTarget::Intensity, s.target());
    }

    #[test]
    fn from_completion_time() {
        let s = Silencer::from_completion_time(Duration::from_secs(1), Duration::from_secs(1));
        assert_eq!(Duration::from_secs(1), s.completion_time_intensity(),);
        assert_eq!(Duration::from_secs(1), s.completion_time_phase());
        assert_eq!(SilencerTarget::Intensity, s.target());
    }

    #[test]
    fn from_completion_time_mul() {
        let s = Silencer::from_completion_time(Duration::from_secs(1), Duration::from_secs(1));
        let s = s * 2;
        assert_eq!(s.completion_time_intensity(), Duration::from_secs(2));
        assert_eq!(s.completion_time_phase(), Duration::from_secs(2));
        assert_eq!(SilencerTarget::Intensity, s.target());
    }

    #[test]
    fn from_completion_time_div() {
        let s = Silencer::from_completion_time(Duration::from_secs(2), Duration::from_secs(2));
        let s = s / 2;
        assert_eq!(Duration::from_secs(1), s.completion_time_intensity(),);
        assert_eq!(Duration::from_secs(1), s.completion_time_phase());
        assert_eq!(SilencerTarget::Intensity, s.target());
    }
}
