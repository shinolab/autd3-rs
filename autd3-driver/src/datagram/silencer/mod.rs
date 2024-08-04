mod fixed_complition_steps;
mod fixed_update_rate;

use std::num::NonZeroU8;

pub use fixed_complition_steps::FixedCompletionSteps;
pub use fixed_update_rate::FixedUpdateRate;

use crate::firmware::operation::SilencerTarget;

use derive_more::Deref;

#[derive(Debug, Clone, Copy, Deref)]
pub struct Silencer<T> {
    #[deref]
    internal: T,
}

pub type SilencerFixedCompletionSteps = Silencer<FixedCompletionSteps>;
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

    pub const fn from_completion_steps(
        steps_intensity: NonZeroU8,
        steps_phase: NonZeroU8,
    ) -> Silencer<FixedCompletionSteps> {
        Silencer {
            internal: FixedCompletionSteps {
                completion_steps_intensity: steps_intensity,
                completion_steps_phase: steps_phase,
                strict_mode: true,
                target: SilencerTarget::Intensity,
            },
        }
    }

    pub const fn disable() -> Silencer<FixedCompletionSteps> {
        Silencer {
            internal: FixedCompletionSteps {
                completion_steps_intensity: NonZeroU8::MIN,
                completion_steps_phase: NonZeroU8::MIN,
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
        assert_eq!(1, s.completion_steps_intensity().get());
        assert_eq!(1, s.completion_steps_phase().get());
        assert!(s.strict_mode());
        assert_eq!(SilencerTarget::Intensity, s.target());
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn from_update_rate() {
        let s = unsafe {
            Silencer::from_update_rate(NonZeroU8::new_unchecked(1), NonZeroU8::new_unchecked(2))
        };
        assert_eq!(1, s.update_rate_intensity().get());
        assert_eq!(2, s.update_rate_phase().get());
        assert_eq!(SilencerTarget::Intensity, s.target());
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn from_completion_time() {
        let s = unsafe {
            Silencer::from_completion_steps(
                NonZeroU8::new_unchecked(1),
                NonZeroU8::new_unchecked(2),
            )
        };
        assert_eq!(1, s.completion_steps_intensity().get());
        assert_eq!(2, s.completion_steps_phase().get());
        assert_eq!(SilencerTarget::Intensity, s.target());
    }
}
