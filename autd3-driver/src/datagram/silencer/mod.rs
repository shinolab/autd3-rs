mod fixed_complition_steps;
mod fixed_complition_time;
mod fixed_update_rate;

pub use fixed_complition_steps::FixedCompletionSteps;
pub use fixed_complition_time::FixedCompletionTime;
pub use fixed_update_rate::FixedUpdateRate;

use derive_more::{Div, Mul};

use crate::firmware::operation::Target;

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
                target: Target::Intensity,
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
                target: Target::Intensity,
            },
        }
    }

    pub const fn from_completion_time(
        time_intensity: std::time::Duration,
        time_phase: std::time::Duration,
    ) -> Silencer<FixedCompletionTime> {
        Silencer {
            internal: FixedCompletionTime {
                time_intensity,
                time_phase,
                strict_mode: true,
                target: Target::Intensity,
            },
        }
    }

    pub const fn disable() -> Silencer<FixedCompletionSteps> {
        Silencer {
            internal: FixedCompletionSteps {
                steps_intensity: 1,
                steps_phase: 1,
                strict_mode: true,
                target: Target::Intensity,
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
        assert_eq!(s.update_rate_intensity(), 1);
        assert_eq!(s.update_rate_phase(), 2);
    }

    #[test]
    fn from_completion_steps_mul() {
        let s = Silencer::from_completion_steps(1, 1);
        let s = s * 2;
        assert_eq!(s.completion_steps_intensity(), 2);
        assert_eq!(s.completion_steps_phase(), 2);
    }

    #[test]
    fn from_completion_steps_div() {
        let s = Silencer::from_completion_steps(2, 2);
        let s = s / 2;
        assert_eq!(s.completion_steps_intensity(), 1);
        assert_eq!(s.completion_steps_phase(), 1);
    }

    #[test]
    fn from_completion_time() {
        let s = Silencer::from_completion_time(
            std::time::Duration::from_secs(1),
            std::time::Duration::from_secs(1),
        );
        assert_eq!(
            s.completion_time_intensity(),
            std::time::Duration::from_secs(1)
        );
        assert_eq!(s.completion_time_phase(), std::time::Duration::from_secs(1));
    }

    #[test]
    fn from_completion_time_mul() {
        let s = Silencer::from_completion_time(
            std::time::Duration::from_secs(1),
            std::time::Duration::from_secs(1),
        );
        let s = s * 2;
        assert_eq!(
            s.completion_time_intensity(),
            std::time::Duration::from_secs(2)
        );
        assert_eq!(s.completion_time_phase(), std::time::Duration::from_secs(2));
    }

    #[test]
    fn from_completion_time_div() {
        let s = Silencer::from_completion_time(
            std::time::Duration::from_secs(2),
            std::time::Duration::from_secs(2),
        );
        let s = s / 2;
        assert_eq!(
            s.completion_time_intensity(),
            std::time::Duration::from_secs(1)
        );
        assert_eq!(s.completion_time_phase(), std::time::Duration::from_secs(1));
    }
}
