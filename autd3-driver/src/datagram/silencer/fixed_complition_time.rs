use std::{num::NonZeroU8, time::Duration};

use autd3_derive::Builder;

use crate::{
    datagram::*,
    defined::ULTRASOUND_FREQ,
    firmware::operation::{SilencerFixedCompletionStepsOp, SilencerTarget},
};

#[derive(Debug, Clone, Copy, Builder)]
pub struct FixedCompletionTime {
    #[get]
    pub(super) completion_time_intensity: Duration,
    #[get]
    pub(super) completion_time_phase: Duration,
    #[get]
    pub(super) strict_mode: bool,
    #[get]
    pub(super) target: SilencerTarget,
}

impl Default for Silencer<FixedCompletionTime> {
    fn default() -> Self {
        Self {
            internal: FixedCompletionTime {
                completion_time_intensity: Silencer::DEFAULT_COMPLETION_TIME_INTENSITY,
                completion_time_phase: Silencer::DEFAULT_COMPLETION_TIME_PHASE,
                strict_mode: true,
                target: SilencerTarget::Intensity,
            },
        }
    }
}

impl Silencer<FixedCompletionTime> {
    pub const fn with_strict_mode(mut self, strict_mode: bool) -> Self {
        self.internal.strict_mode = strict_mode;
        self
    }

    pub const fn with_target(mut self, target: SilencerTarget) -> Self {
        self.internal.target = target;
        self
    }
}

impl Silencer<FixedCompletionTime> {
    pub fn is_valid<T: WithSampling>(&self, target: &T) -> bool {
        if !self.internal.strict_mode {
            return true;
        }

        let intensity_freq_div = target
            .sampling_config_intensity()
            .map_or(Duration::MAX, |c| c.period());
        let phase_freq_div = target
            .sampling_config_phase()
            .map_or(Duration::MAX, |c| c.period());

        self.completion_time_intensity() <= intensity_freq_div
            && self.completion_time_phase() <= phase_freq_div
    }
}

#[derive(Debug)]
pub struct SilencerFixedCompletionTimeOpGenerator {
    steps_intensity: NonZeroU8,
    steps_phase: NonZeroU8,
    strict_mode: bool,
    target: SilencerTarget,
}

impl OperationGenerator for SilencerFixedCompletionTimeOpGenerator {
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

impl Datagram for Silencer<FixedCompletionTime> {
    type G = SilencerFixedCompletionTimeOpGenerator;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation_generator(self, _: &Geometry) -> Result<Self::G, AUTDInternalError> {
        let validate = |value: Duration| {
            const NANOSEC: u128 = 1_000_000_000;
            let v = value.as_nanos() * ULTRASOUND_FREQ.hz() as u128;
            let v = if v % NANOSEC == 0 {
                v / NANOSEC
            } else {
                return Err(AUTDInternalError::InvalidSilencerCompletionTime(value));
            };
            if v == 0 || v > u8::MAX as _ {
                return Err(AUTDInternalError::SilencerCompletionTimeOutOfRange(value));
            }
            Ok(unsafe { NonZeroU8::new_unchecked(v as _) })
        };
        Ok(SilencerFixedCompletionTimeOpGenerator {
            steps_intensity: validate(self.internal.completion_time_intensity)?,
            steps_phase: validate(self.internal.completion_time_phase)?,
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
    use std::sync::Arc;

    use gain::tests::TestGain;
    use modulation::tests::TestModulation;

    use crate::{
        defined::ULTRASOUND_PERIOD,
        derive::LoopBehavior,
        geometry::{tests::create_geometry, Vector3},
    };

    use super::*;

    #[test]
    #[cfg_attr(miri, ignore)]
    fn fixed_completion_time() {
        let d =
            Silencer::from_completion_time(Duration::from_micros(25), Duration::from_micros(50));
        assert_eq!(d.completion_time_intensity(), Duration::from_micros(25));
        assert_eq!(d.completion_time_phase(), Duration::from_micros(50));
        assert!(d.strict_mode());
    }

    #[rstest::rstest]
    #[test]
    #[case(
        AUTDInternalError::SilencerCompletionTimeOutOfRange(Duration::from_micros(0)),
        Duration::from_micros(0),
        Duration::from_micros(25)
    )]
    #[case(
        AUTDInternalError::SilencerCompletionTimeOutOfRange(Duration::from_micros(25 * 256)),
        Duration::from_micros(25 * 256),
        Duration::from_micros(25)
    )]
    #[case(
        AUTDInternalError::SilencerCompletionTimeOutOfRange(Duration::from_micros(0)),
        Duration::from_micros(25),
        Duration::from_micros(0)
    )]
    #[case(
        AUTDInternalError::SilencerCompletionTimeOutOfRange(Duration::from_micros(25 * 256)),
        Duration::from_micros(25),
        Duration::from_micros(25 * 256),
    )]
    #[case(
        AUTDInternalError::InvalidSilencerCompletionTime(Duration::from_micros(26)),
        Duration::from_micros(26),
        Duration::from_micros(50)
    )]
    #[case(
        AUTDInternalError::InvalidSilencerCompletionTime(Duration::from_micros(51)),
        Duration::from_micros(25),
        Duration::from_micros(51)
    )]
    #[cfg_attr(miri, ignore)]
    fn invalid_time(
        #[case] expected: AUTDInternalError,
        #[case] time_intensity: Duration,
        #[case] time_phase: Duration,
    ) {
        let geometry = create_geometry(1, 1);
        assert_eq!(
            expected,
            Silencer::from_completion_time(time_intensity, time_phase)
                .operation_generator(&geometry)
                .unwrap_err()
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(true, 10, 10, true, FociSTM::new(SamplingConfig::new(10).unwrap(), [Vector3::zeros()]).unwrap())]
    #[case(false, 11, 10, true, FociSTM::new(SamplingConfig::new(10).unwrap(), [Vector3::zeros()]).unwrap())]
    #[case(false, 10, 11, true, FociSTM::new(SamplingConfig::new(10).unwrap(), [Vector3::zeros()]).unwrap())]
    #[case(true, 11, 10, false, FociSTM::new(SamplingConfig::new(10).unwrap(), [Vector3::zeros()]).unwrap())]
    #[case(true, 10, 11, false, FociSTM::new(SamplingConfig::new(10).unwrap(), [Vector3::zeros()]).unwrap())]
    #[case(true, 10, 10, true, GainSTM::new(SamplingConfig::new(10).unwrap(), [TestGain{ data: Default::default(), err: None }]).unwrap())]
    #[case(false, 11, 10, true, GainSTM::new(SamplingConfig::new(10).unwrap(), [TestGain{ data: Default::default(), err: None }]).unwrap())]
    #[case(false, 10, 11, true, GainSTM::new(SamplingConfig::new(10).unwrap(), [TestGain{ data: Default::default(), err: None }]).unwrap())]
    #[case(true, 11, 10, false, GainSTM::new(SamplingConfig::new(10).unwrap(), [TestGain{ data: Default::default(), err: None }]).unwrap())]
    #[case(true, 10, 11, false, GainSTM::new(SamplingConfig::new(10).unwrap(), [TestGain{ data: Default::default(), err: None }]).unwrap())]
    #[case(true, 10, 10, true, TestModulation { config: SamplingConfig::new(10).unwrap(), buf: Arc::new(Vec::new()), loop_behavior: LoopBehavior::infinite() })]
    #[case(false, 11, 10, true, TestModulation { config: SamplingConfig::new(10).unwrap(), buf: Arc::new(Vec::new()), loop_behavior: LoopBehavior::infinite() })]
    #[case(true, 10, 11, true, TestModulation { config: SamplingConfig::new(10).unwrap(), buf: Arc::new(Vec::new()), loop_behavior: LoopBehavior::infinite() })]
    #[case(true, 11, 10, false, TestModulation { config: SamplingConfig::new(10).unwrap(), buf: Arc::new(Vec::new()), loop_behavior: LoopBehavior::infinite() })]
    #[case(true, 10, 11, false, TestModulation { config: SamplingConfig::new(10).unwrap(), buf: Arc::new(Vec::new()), loop_behavior: LoopBehavior::infinite() })]
    #[cfg_attr(miri, ignore)]
    fn is_valid(
        #[case] expect: bool,
        #[case] intensity: u32,
        #[case] phase: u32,
        #[case] strict: bool,
        #[case] target: impl WithSampling,
    ) {
        let s = Silencer::from_completion_time(
            intensity * ULTRASOUND_PERIOD,
            phase * ULTRASOUND_PERIOD,
        )
        .with_strict_mode(strict);
        assert_eq!(expect, s.is_valid(&target));
    }
}
