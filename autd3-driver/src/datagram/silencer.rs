use std::{num::NonZeroU16, time::Duration};

use autd3_derive::Builder;

use crate::{
    defined::ULTRASOUND_PERIOD,
    error::AUTDInternalError,
    firmware::{
        fpga::{
            SamplingConfig, SilencerTarget, SILENCER_STEPS_INTENSITY_DEFAULT,
            SILENCER_STEPS_PHASE_DEFAULT,
        },
        operation::{
            NullOp, OperationGenerator, SilencerFixedCompletionStepsOp, SilencerFixedUpdateRateOp,
        },
    },
    geometry::{Device, Geometry},
};

use super::Datagram;

pub trait WithSampling {
    fn sampling_config_intensity(&self) -> Option<SamplingConfig>;
    fn sampling_config_phase(&self) -> Option<SamplingConfig>;
}

pub trait SilencerConfig: std::fmt::Debug + Clone + Copy {}
impl SilencerConfig for () {}

#[derive(Debug, Clone, Copy, Builder, PartialEq, Eq)]
pub struct FixedCompletionTime {
    #[get]
    pub intensity: Duration,
    #[get]
    pub phase: Duration,
}
impl SilencerConfig for FixedCompletionTime {}

#[derive(Debug, Clone, Copy, Builder, PartialEq, Eq)]
pub struct FixedUpdateRate {
    #[get]
    pub intensity: NonZeroU16,
    #[get]
    pub phase: NonZeroU16,
}
impl SilencerConfig for FixedUpdateRate {}

#[derive(Debug, Clone, Copy, Builder)]
pub struct Silencer<T: SilencerConfig> {
    #[get]
    config: T,
    strict_mode: bool,
    #[get]
    #[set]
    target: SilencerTarget,
}

impl Silencer<()> {
    pub const DEFAULT_COMPLETION_TIME_INTENSITY: Duration =
        Duration::from_micros(25 * SILENCER_STEPS_INTENSITY_DEFAULT as u64);
    pub const DEFAULT_COMPLETION_TIME_PHASE: Duration =
        Duration::from_micros(25 * SILENCER_STEPS_PHASE_DEFAULT as u64);

    pub const fn new<T: SilencerConfig>(config: T) -> Silencer<T> {
        Silencer {
            config,
            strict_mode: true,
            target: SilencerTarget::Intensity,
        }
    }

    pub const fn disable() -> Silencer<FixedCompletionTime> {
        Silencer::new(FixedCompletionTime {
            intensity: ULTRASOUND_PERIOD,
            phase: ULTRASOUND_PERIOD,
        })
    }
}

impl Silencer<FixedUpdateRate> {
    pub const fn strict_mode(&self) -> bool {
        false
    }

    pub const fn is_valid<T: WithSampling>(&self, _target: &T) -> bool {
        true
    }
}

impl Silencer<FixedCompletionTime> {
    pub const fn strict_mode(&self) -> bool {
        self.strict_mode
    }

    pub const fn with_strict_mode(mut self, strict_mode: bool) -> Self {
        self.strict_mode = strict_mode;
        self
    }

    pub fn is_valid<T: WithSampling>(&self, target: &T) -> bool {
        if !self.strict_mode {
            return true;
        }

        let intensity_freq_div = target
            .sampling_config_intensity()
            .map_or(Duration::MAX, |c| c.period());
        let phase_freq_div = target
            .sampling_config_phase()
            .map_or(Duration::MAX, |c| c.period());

        self.config.intensity <= intensity_freq_div && self.config.phase <= phase_freq_div
    }
}

impl Default for Silencer<FixedCompletionTime> {
    fn default() -> Self {
        Silencer::new(FixedCompletionTime {
            intensity: Silencer::DEFAULT_COMPLETION_TIME_INTENSITY,
            phase: Silencer::DEFAULT_COMPLETION_TIME_PHASE,
        })
    }
}

pub struct SilencerOpGenerator<T: SilencerConfig> {
    config: T,
    strict_mode: bool,
    target: SilencerTarget,
}

impl OperationGenerator for SilencerOpGenerator<FixedUpdateRate> {
    type O1 = SilencerFixedUpdateRateOp;
    type O2 = NullOp;

    fn generate(&mut self, _: &Device) -> (Self::O1, Self::O2) {
        (
            Self::O1::new(self.config.intensity, self.config.phase, self.target),
            Self::O2::new(),
        )
    }
}

impl OperationGenerator for SilencerOpGenerator<FixedCompletionTime> {
    type O1 = SilencerFixedCompletionStepsOp;
    type O2 = NullOp;

    fn generate(&mut self, _: &Device) -> (Self::O1, Self::O2) {
        (
            Self::O1::new(
                self.config.intensity,
                self.config.phase,
                self.strict_mode,
                self.target,
            ),
            Self::O2::new(),
        )
    }
}

impl<T: SilencerConfig> Datagram for Silencer<T>
where
    SilencerOpGenerator<T>: OperationGenerator,
{
    type G = SilencerOpGenerator<T>;

    fn operation_generator(self, _: &Geometry) -> Result<Self::G, AUTDInternalError> {
        Ok(Self::G {
            config: self.config,
            strict_mode: self.strict_mode,
            target: self.target,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        datagram::{gain::tests::TestGain, modulation::tests::TestModulation, FociSTM, GainSTM},
        firmware::fpga::LoopBehavior,
        geometry::Vector3,
    };

    use super::*;

    #[test]
    fn disable() {
        let s = Silencer::disable();
        assert_eq!(ULTRASOUND_PERIOD, s.config().intensity());
        assert_eq!(ULTRASOUND_PERIOD, s.config().phase());
        assert!(s.strict_mode());
        assert_eq!(SilencerTarget::Intensity, s.target());
    }

    #[test]
    fn from_update_rate() {
        let s = Silencer::new(FixedUpdateRate {
            intensity: NonZeroU16::new(1).unwrap(),
            phase: NonZeroU16::new(2).unwrap(),
        });
        assert_eq!(1, s.config().intensity().get());
        assert_eq!(2, s.config().phase().get());
        assert_eq!(SilencerTarget::Intensity, s.target());
    }

    #[test]
    fn from_completion_time() {
        let s = Silencer::new(FixedCompletionTime {
            intensity: Duration::from_secs(1),
            phase: Duration::from_secs(1),
        });
        assert_eq!(Duration::from_secs(1), s.config().intensity());
        assert_eq!(Duration::from_secs(1), s.config().phase());
        assert_eq!(SilencerTarget::Intensity, s.target());
    }

    #[test]
    fn fixed_update_rate_strict_mode() {
        let s = Silencer::new(FixedUpdateRate {
            intensity: NonZeroU16::new(1).unwrap(),
            phase: NonZeroU16::new(2).unwrap(),
        });
        assert!(!s.strict_mode());
    }

    #[rstest::rstest]
    #[test]
    #[case(FociSTM::new(
        SamplingConfig::FREQ_4K,
        [Vector3::zeros(), Vector3::zeros()]
    ).unwrap())]
    #[case(GainSTM::new(
        SamplingConfig::FREQ_4K,
        [TestGain{ data: Default::default() }, TestGain{ data: Default::default() }]
    ).unwrap())]
    #[case(TestModulation {
        config: SamplingConfig::FREQ_4K,
        loop_behavior: LoopBehavior::infinite(),
    })]
    fn fixed_update_rate_is_valid(#[case] target: impl WithSampling) {
        let s = Silencer::new(FixedUpdateRate {
            intensity: NonZeroU16::new(1).unwrap(),
            phase: NonZeroU16::new(2).unwrap(),
        });
        assert!(s.is_valid(&target));
    }

    #[rstest::rstest]
    #[test]
    #[case(true, 10, 10, true, FociSTM::new(SamplingConfig::new(10).unwrap(), [Vector3::zeros()]).unwrap())]
    #[case(false, 11, 10, true, FociSTM::new(SamplingConfig::new(10).unwrap(), [Vector3::zeros()]).unwrap())]
    #[case(false, 10, 11, true, FociSTM::new(SamplingConfig::new(10).unwrap(), [Vector3::zeros()]).unwrap())]
    #[case(true, 11, 10, false, FociSTM::new(SamplingConfig::new(10).unwrap(), [Vector3::zeros()]).unwrap())]
    #[case(true, 10, 11, false, FociSTM::new(SamplingConfig::new(10).unwrap(), [Vector3::zeros()]).unwrap())]
    #[case(true, 10, 10, true, GainSTM::new(SamplingConfig::new(10).unwrap(), [TestGain{ data: Default::default() }]).unwrap())]
    #[case(false, 11, 10, true, GainSTM::new(SamplingConfig::new(10).unwrap(), [TestGain{ data: Default::default() }]).unwrap())]
    #[case(false, 10, 11, true, GainSTM::new(SamplingConfig::new(10).unwrap(), [TestGain{ data: Default::default() }]).unwrap())]
    #[case(true, 11, 10, false, GainSTM::new(SamplingConfig::new(10).unwrap(), [TestGain{ data: Default::default() }]).unwrap())]
    #[case(true, 10, 11, false, GainSTM::new(SamplingConfig::new(10).unwrap(), [TestGain{ data: Default::default() }]).unwrap())]
    #[case(true, 10, 10, true, TestModulation { config: SamplingConfig::new(10).unwrap(), loop_behavior: LoopBehavior::infinite() })]
    #[case(false, 11, 10, true, TestModulation { config: SamplingConfig::new(10).unwrap(), loop_behavior: LoopBehavior::infinite() })]
    #[case(true, 10, 11, true, TestModulation { config: SamplingConfig::new(10).unwrap(), loop_behavior: LoopBehavior::infinite() })]
    #[case(true, 11, 10, false, TestModulation { config: SamplingConfig::new(10).unwrap(), loop_behavior: LoopBehavior::infinite() })]
    #[case(true, 10, 11, false, TestModulation { config: SamplingConfig::new(10).unwrap(), loop_behavior: LoopBehavior::infinite() })]
    fn fixed_completion_time_is_valid(
        #[case] expect: bool,
        #[case] intensity: u32,
        #[case] phase: u32,
        #[case] strict: bool,
        #[case] target: impl WithSampling,
    ) {
        let s = Silencer::new(FixedCompletionTime {
            intensity: intensity * ULTRASOUND_PERIOD,
            phase: phase * ULTRASOUND_PERIOD,
        })
        .with_strict_mode(strict);
        assert_eq!(expect, s.is_valid(&target));
    }
}
