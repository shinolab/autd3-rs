use std::num::NonZeroU8;

use crate::{
    datagram::*,
    firmware::{
        fpga::{SILENCER_STEPS_INTENSITY_DEFAULT, SILENCER_STEPS_PHASE_DEFAULT},
        operation::{SilencerFixedCompletionStepsOp, SilencerTarget},
    },
};

#[derive(Debug, Clone, Copy)]
pub struct FixedCompletionSteps {
    pub(super) steps_intensity: NonZeroU8,
    pub(super) steps_phase: NonZeroU8,
    pub(super) strict_mode: bool,
    pub(super) target: SilencerTarget,
}

impl Default for Silencer<FixedCompletionSteps> {
    fn default() -> Self {
        Self {
            internal: FixedCompletionSteps {
                steps_intensity: unsafe {
                    NonZeroU8::new_unchecked(SILENCER_STEPS_INTENSITY_DEFAULT)
                },
                steps_phase: unsafe { NonZeroU8::new_unchecked(SILENCER_STEPS_PHASE_DEFAULT) },
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

    pub const fn with_target(mut self, target: SilencerTarget) -> Self {
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

impl Silencer<FixedCompletionSteps> {
    pub fn is_valid<T: WithSampling>(&self, target: &T) -> bool {
        if !self.internal.strict_mode {
            return true;
        }

        let intensity_freq_div = target
            .sampling_config_intensity()
            .map_or(0xFFFF, |c| c.division());
        let phase_freq_div = target
            .sampling_config_phase()
            .map_or(0xFFFF, |c| c.division());

        self.completion_steps_intensity() as u16 <= intensity_freq_div
            && self.completion_steps_phase() as u16 <= phase_freq_div
    }
}

#[derive(Debug)]
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
    use std::{num::NonZeroU16, sync::Arc};

    use gain::tests::TestGain;
    use modulation::tests::TestModulation;

    use crate::{derive::LoopBehavior, geometry::Vector3};

    use super::*;

    #[test]
    #[cfg_attr(miri, ignore)]
    fn fixed_completion_steps() {
        let d = unsafe {
            Silencer::from_completion_steps(
                NonZeroU8::new_unchecked(1),
                NonZeroU8::new_unchecked(2),
            )
        };
        assert_eq!(1, d.completion_steps_intensity());
        assert_eq!(2, d.completion_steps_phase());
        assert!(d.strict_mode());
    }

    #[rstest::rstest]
    #[test]
    #[case(true, 10, 10, true, FociSTM::new(SamplingConfig::new(NonZeroU16::new(10).unwrap()), [Vector3::zeros()]).unwrap())]
    #[case(false, 11, 10, true, FociSTM::new(SamplingConfig::new(NonZeroU16::new(10).unwrap()), [Vector3::zeros()]).unwrap())]
    #[case(false, 10, 11, true, FociSTM::new(SamplingConfig::new(NonZeroU16::new(10).unwrap()), [Vector3::zeros()]).unwrap())]
    #[case(true, 11, 10, false, FociSTM::new(SamplingConfig::new(NonZeroU16::new(10).unwrap()), [Vector3::zeros()]).unwrap())]
    #[case(true, 10, 11, false, FociSTM::new(SamplingConfig::new(NonZeroU16::new(10).unwrap()), [Vector3::zeros()]).unwrap())]
    #[case(true, 10, 10, true, GainSTM::new(SamplingConfig::new(NonZeroU16::new(10).unwrap()), [TestGain{ data: Default::default(), err: None }]).unwrap())]
    #[case(false, 11, 10, true, GainSTM::new(SamplingConfig::new(NonZeroU16::new(10).unwrap()), [TestGain{ data: Default::default(), err: None }]).unwrap())]
    #[case(false, 10, 11, true, GainSTM::new(SamplingConfig::new(NonZeroU16::new(10).unwrap()), [TestGain{ data: Default::default(), err: None }]).unwrap())]
    #[case(true, 11, 10, false, GainSTM::new(SamplingConfig::new(NonZeroU16::new(10).unwrap()), [TestGain{ data: Default::default(), err: None }]).unwrap())]
    #[case(true, 10, 11, false, GainSTM::new(SamplingConfig::new(NonZeroU16::new(10).unwrap()), [TestGain{ data: Default::default(), err: None }]).unwrap())]
    #[case(true, 10, 10, true, TestModulation { config: SamplingConfig::new(NonZeroU16::new(10).unwrap()), buf: Arc::new(Vec::new()), loop_behavior: LoopBehavior::infinite() })]
    #[case(false, 11, 10, true, TestModulation { config: SamplingConfig::new(NonZeroU16::new(10).unwrap()), buf: Arc::new(Vec::new()), loop_behavior: LoopBehavior::infinite() })]
    #[case(true, 10, 11, true, TestModulation { config: SamplingConfig::new(NonZeroU16::new(10).unwrap()), buf: Arc::new(Vec::new()), loop_behavior: LoopBehavior::infinite() })]
    #[case(true, 11, 10, false, TestModulation { config: SamplingConfig::new(NonZeroU16::new(10).unwrap()), buf: Arc::new(Vec::new()), loop_behavior: LoopBehavior::infinite() })]
    #[case(true, 10, 11, false, TestModulation { config: SamplingConfig::new(NonZeroU16::new(10).unwrap()), buf: Arc::new(Vec::new()), loop_behavior: LoopBehavior::infinite() })]
    #[cfg_attr(miri, ignore)]
    fn is_valid(
        #[case] expect: bool,
        #[case] intensity: u8,
        #[case] phase: u8,
        #[case] strict: bool,
        #[case] target: impl WithSampling,
    ) {
        let s = Silencer::from_completion_steps(
            NonZeroU8::new(intensity).unwrap(),
            NonZeroU8::new(phase).unwrap(),
        )
        .with_strict_mode(strict);
        assert_eq!(expect, s.is_valid(&target));
    }
}
