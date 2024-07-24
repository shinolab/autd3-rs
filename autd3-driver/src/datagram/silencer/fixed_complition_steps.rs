use std::num::NonZeroU8;

use crate::firmware::{
    fpga::{SILENCER_STEPS_INTENSITY_DEFAULT, SILENCER_STEPS_PHASE_DEFAULT},
    operation::SilencerFixedCompletionStepsOp,
};
use crate::{datagram::*, firmware::operation::SilencerTarget};

#[derive(Debug, Clone, Copy)]
pub struct FixedCompletionSteps {
    pub(super) steps_intensity: NonZeroU8,
    pub(super) steps_phase: NonZeroU8,
    pub(super) strict_mode: bool,
    pub(super) target: SilencerTarget,
}

impl Default for Silencer<FixedCompletionSteps> {
    fn default() -> Self {
        Silencer {
            internal: FixedCompletionSteps {
                steps_intensity: SILENCER_STEPS_INTENSITY_DEFAULT,
                steps_phase: SILENCER_STEPS_PHASE_DEFAULT,
                strict_mode: true,
                target: SilencerTarget::Intensity,
            },
        }
    }
}

impl Silencer<FixedCompletionSteps> {
    pub const fn with_strict_mode(mut self, strict_mode: bool) -> Self {
        self.internal.strict_mode = strict_mode;
        self
    }

    pub const fn with_taget(mut self, target: SilencerTarget) -> Self {
        self.internal.target = target;
        self
    }

    pub const fn completion_steps_intensity(&self) -> u8 {
        self.internal.steps_intensity.get()
    }

    pub const fn completion_steps_phase(&self) -> u8 {
        self.internal.steps_phase.get()
    }

    pub const fn strict_mode(&self) -> bool {
        self.internal.strict_mode
    }

    pub const fn target(&self) -> SilencerTarget {
        self.internal.target
    }
}

macro_rules! impl_saturating_op {
    ($op:ident) => {
        impl Silencer<FixedCompletionSteps> {
            pub const fn $op(self, v: u8) -> Self {
                let steps_intensity = match self.completion_steps_intensity().$op(v) {
                    0 => 1,
                    v => v,
                };
                let steps_phase = match self.completion_steps_phase().$op(v) {
                    0 => 1,
                    v => v,
                };
                Self {
                    internal: FixedCompletionSteps {
                        steps_intensity: unsafe { NonZeroU8::new_unchecked(steps_intensity) },
                        steps_phase: unsafe { NonZeroU8::new_unchecked(steps_phase) },
                        strict_mode: self.strict_mode(),
                        target: self.target(),
                    },
                }
            }
        }
    };
}
impl_saturating_op!(saturating_add);
impl_saturating_op!(saturating_sub);
impl_saturating_op!(saturating_mul);
impl_saturating_op!(saturating_div);

macro_rules! impl_checked_op {
    ($op:ident) => {
        impl Silencer<FixedCompletionSteps> {
            pub const fn $op(self, v: u8) -> Option<Self> {
                let (steps_intensity, steps_phase) = match (
                    self.completion_steps_intensity().$op(v),
                    self.completion_steps_phase().$op(v),
                ) {
                    (None, _) | (_, None) | (Some(0), _) | (_, Some(0)) => return None,
                    (Some(i), Some(p)) => (i, p),
                };
                Some(Self {
                    internal: FixedCompletionSteps {
                        steps_intensity: unsafe { NonZeroU8::new_unchecked(steps_intensity) },
                        steps_phase: unsafe { NonZeroU8::new_unchecked(steps_phase) },
                        strict_mode: self.strict_mode(),
                        target: self.target(),
                    },
                })
            }
        }
    };
}
impl_checked_op!(checked_add);
impl_checked_op!(checked_sub);
impl_checked_op!(checked_mul);
impl_checked_op!(checked_div);

pub struct SilencerFixedCompletionStepsOpGenerator {
    steps_intensity: NonZeroU8,
    steps_phase: NonZeroU8,
    strict_mode: bool,
    target: SilencerTarget,
}

impl OperationGenerator for SilencerFixedCompletionStepsOpGenerator {
    type O1 = SilencerFixedCompletionStepsOp;
    type O2 = NullOp;

    fn generate(&self, _: &Device) -> (Self::O1, Self::O2) {
        (
            Self::O1::new(
                self.steps_intensity,
                self.steps_phase,
                self.strict_mode,
                self.target,
            ),
            Self::O2::default(),
        )
    }
}

impl Datagram for Silencer<FixedCompletionSteps> {
    type G = SilencerFixedCompletionStepsOpGenerator;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation_generator(self, _: &Geometry) -> Result<Self::G, AUTDInternalError> {
        Ok(SilencerFixedCompletionStepsOpGenerator {
            steps_intensity: self.internal.steps_intensity,
            steps_phase: self.internal.steps_phase,
            strict_mode: self.internal.strict_mode,
            target: self.internal.target,
        })
    }

    fn parallel_threshold(&self) -> Option<usize> {
        Some(usize::MAX)
    }

    #[tracing::instrument(level = "debug", skip(_geometry))]
    // GRCOV_EXCL_START
    fn trace(&self, _geometry: &Geometry) {
        tracing::debug!("{}", tynm::type_name::<Self>());
    }
    // GRCOV_EXCL_STOP
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[test]
    #[case(0x02, NonZeroU8::new(0x01).unwrap(), 0x01)]
    #[case(0xFF, NonZeroU8::new(0x01).unwrap(), 0xFE)]
    #[case(0xFF, NonZeroU8::new(0x01).unwrap(), 0xFF)]
    #[cfg_attr(miri, ignore)]
    fn saturating_add(#[case] expect: u8, #[case] value: NonZeroU8, #[case] add: u8) {
        let d = Silencer::from_completion_steps(value, value);
        let d = d.saturating_add(add);
        assert_eq!(expect, d.completion_steps_intensity());
        assert_eq!(expect, d.completion_steps_phase());
    }

    #[rstest::rstest]
    #[test]
    #[case(0xFE, NonZeroU8::new(0xFF).unwrap(), 0x01)]
    #[case(0x01, NonZeroU8::new(0xFF).unwrap(), 0xFE)]
    #[case(0x01, NonZeroU8::new(0xFF).unwrap(), 0xFF)]
    #[cfg_attr(miri, ignore)]
    fn saturating_sub(#[case] expect: u8, #[case] value: NonZeroU8, #[case] add: u8) {
        let d = Silencer::from_completion_steps(value, value);
        let d = d.saturating_sub(add);
        assert_eq!(expect, d.completion_steps_intensity());
        assert_eq!(expect, d.completion_steps_phase());
    }

    #[rstest::rstest]
    #[test]
    #[case(0x7F, NonZeroU8::new(0x7F).unwrap(), 0x01)]
    #[case(0xFE, NonZeroU8::new(0x7F).unwrap(), 0x02)]
    #[case(0xFF, NonZeroU8::new(0x7F).unwrap(), 0x03)]
    #[cfg_attr(miri, ignore)]
    fn saturating_mul(#[case] expect: u8, #[case] value: NonZeroU8, #[case] add: u8) {
        let d = Silencer::from_completion_steps(value, value);
        let d = d.saturating_mul(add);
        assert_eq!(expect, d.completion_steps_intensity());
        assert_eq!(expect, d.completion_steps_phase());
    }

    #[rstest::rstest]
    #[test]
    #[case(0x80, NonZeroU8::new(0x80).unwrap(), 0x01)]
    #[case(0x01, NonZeroU8::new(0x80).unwrap(), 0x80)]
    #[case(0x01, NonZeroU8::new(0x80).unwrap(), 0x81)]
    #[cfg_attr(miri, ignore)]
    fn saturating_div(#[case] expect: u8, #[case] value: NonZeroU8, #[case] add: u8) {
        let d = Silencer::from_completion_steps(value, value);
        let d = d.saturating_div(add);
        assert_eq!(expect, d.completion_steps_intensity());
        assert_eq!(expect, d.completion_steps_phase());
    }

    #[rstest::rstest]
    #[test]
    #[case(Some(0x02), NonZeroU8::new(0x01).unwrap(), 0x01)]
    #[case(Some(0xFF), NonZeroU8::new(0x01).unwrap(), 0xFE)]
    #[case(None, NonZeroU8::new(0x01).unwrap(), 0xFF)]
    #[cfg_attr(miri, ignore)]
    fn checked_add(#[case] expect: Option<u8>, #[case] value: NonZeroU8, #[case] add: u8) {
        let d = Silencer::from_completion_steps(value, value);
        let d = d.checked_add(add);
        assert_eq!(expect, d.map(|d| d.completion_steps_intensity()));
        assert_eq!(expect, d.map(|d| d.completion_steps_phase()));
    }

    #[rstest::rstest]
    #[test]
    #[case(Some(0xFE), NonZeroU8::new(0xFF).unwrap(), 0x01)]
    #[case(Some(0x01), NonZeroU8::new(0xFF).unwrap(), 0xFE)]
    #[case(None, NonZeroU8::new(0xFF).unwrap(), 0xFF)]
    #[cfg_attr(miri, ignore)]
    fn checked_sub(#[case] expect: Option<u8>, #[case] value: NonZeroU8, #[case] add: u8) {
        let d = Silencer::from_completion_steps(value, value);
        let d = d.checked_sub(add);
        assert_eq!(expect, d.map(|d| d.completion_steps_intensity()));
        assert_eq!(expect, d.map(|d| d.completion_steps_phase()));
    }

    #[rstest::rstest]
    #[test]
    #[case(Some(0x7F), NonZeroU8::new(0x7F).unwrap(), 0x01)]
    #[case(Some(0xFE), NonZeroU8::new(0x7F).unwrap(), 0x02)]
    #[case(None, NonZeroU8::new(0x7F).unwrap(), 0x03)]
    #[cfg_attr(miri, ignore)]
    fn checked_mul(#[case] expect: Option<u8>, #[case] value: NonZeroU8, #[case] add: u8) {
        let d = Silencer::from_completion_steps(value, value);
        let d = d.checked_mul(add);
        assert_eq!(expect, d.map(|d| d.completion_steps_intensity()));
        assert_eq!(expect, d.map(|d| d.completion_steps_phase()));
    }

    #[rstest::rstest]
    #[test]
    #[case(Some(0x80), NonZeroU8::new(0x80).unwrap(), 0x01)]
    #[case(Some(0x01), NonZeroU8::new(0x80).unwrap(), 0x80)]
    #[case(None, NonZeroU8::new(0x80).unwrap(), 0x81)]
    #[cfg_attr(miri, ignore)]
    fn checked_div(#[case] expect: Option<u8>, #[case] value: NonZeroU8, #[case] add: u8) {
        let d = Silencer::from_completion_steps(value, value);
        let d = d.checked_div(add);
        assert_eq!(expect, d.map(|d| d.completion_steps_intensity()));
        assert_eq!(expect, d.map(|d| d.completion_steps_phase()));
    }
}
