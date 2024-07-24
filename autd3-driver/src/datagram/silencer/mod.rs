mod fixed_complition_time;
mod fixed_update_rate;

use std::{num::NonZeroU8, time::Duration};

pub use fixed_complition_time::FixedCompletionTime;
pub use fixed_update_rate::FixedUpdateRate;

use crate::{defined::ULTRASOUND_PERIOD, firmware::operation::SilencerTarget};

#[derive(Debug, Clone, Copy)]
pub struct Silencer<T> {
    internal: T,
}

pub type SilencerFixedCompletionTime = Silencer<FixedCompletionTime>;
pub type SilencerFixedUpdateRate = Silencer<FixedUpdateRate>;

impl Silencer<()> {
    pub const fn from_update_rate(
        update_rate_intensity: NonZeroU8,
        update_rate_phase: NonZeroU8,
    ) -> Silencer<FixedUpdateRate> {
        Silencer {
            internal: FixedUpdateRate {
                update_rate_intensity,
                update_rate_phase,
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

    pub const fn disable() -> Silencer<FixedCompletionTime> {
        Silencer {
            internal: FixedCompletionTime {
                time_intensity: ULTRASOUND_PERIOD,
                time_phase: ULTRASOUND_PERIOD,
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
    #[cfg_attr(miri, ignore)]
    fn disable() {
        let s = Silencer::disable();
        assert_eq!(ULTRASOUND_PERIOD, s.completion_time_intensity());
        assert_eq!(ULTRASOUND_PERIOD, s.completion_time_phase());
        assert!(s.strict_mode());
        assert_eq!(SilencerTarget::Intensity, s.target());
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn from_update_rate() {
        let s = unsafe {
            Silencer::from_update_rate(NonZeroU8::new_unchecked(1), NonZeroU8::new_unchecked(2))
        };
        assert_eq!(1, s.update_rate_intensity());
        assert_eq!(2, s.update_rate_phase());
        assert_eq!(SilencerTarget::Intensity, s.target());
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn from_completion_time() {
        let s = Silencer::from_completion_time(Duration::from_secs(1), Duration::from_secs(1));
        assert_eq!(Duration::from_secs(1), s.completion_time_intensity(),);
        assert_eq!(Duration::from_secs(1), s.completion_time_phase());
        assert_eq!(SilencerTarget::Intensity, s.target());
    }
}
