use std::{num::NonZeroU16, time::Duration};

use autd3_derive::Builder;

use crate::{
    defined::ULTRASOUND_PERIOD,
    derive::{Device, Geometry, DEFAULT_TIMEOUT},
    error::AUTDInternalError,
    firmware::{
        fpga::{SilencerTarget, SILENCER_STEPS_INTENSITY_DEFAULT, SILENCER_STEPS_PHASE_DEFAULT},
        operation::{
            NullOp, OperationGenerator, SilencerFixedCompletionStepsOp, SilencerFixedUpdateRateOp,
        },
    },
};

use super::{Datagram, WithSampling};

pub trait SilencerConfig: std::fmt::Debug + Clone + Copy {}
impl SilencerConfig for () {}

#[derive(Debug, Clone, Copy, Builder)]
pub struct FixedCompletionTime {
    #[get]
    pub intensity: Duration,
    #[get]
    pub phase: Duration,
}
impl SilencerConfig for FixedCompletionTime {}

#[derive(Debug, Clone, Copy, Builder)]
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

#[cfg(feature = "capi")]
impl Default for Silencer<FixedUpdateRate> {
    fn default() -> Self {
        Silencer::new(FixedUpdateRate {
            intensity: NonZeroU16::MIN,
            phase: NonZeroU16::MIN,
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

    fn generate(&self, _: &Device) -> (Self::O1, Self::O2) {
        (
            Self::O1::new(self.config.intensity, self.config.phase, self.target),
            Self::O2::default(),
        )
    }
}

impl OperationGenerator for SilencerOpGenerator<FixedCompletionTime> {
    type O1 = SilencerFixedCompletionStepsOp;
    type O2 = NullOp;

    fn generate(&self, _: &Device) -> (Self::O1, Self::O2) {
        (
            Self::O1::new(
                self.config.intensity,
                self.config.phase,
                self.strict_mode,
                self.target,
            ),
            Self::O2::default(),
        )
    }
}

impl<T: SilencerConfig> Datagram for Silencer<T>
where
    SilencerOpGenerator<T>: OperationGenerator,
{
    type G = SilencerOpGenerator<T>;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation_generator(self, _: &Geometry) -> Result<Self::G, AUTDInternalError> {
        Ok(Self::G {
            config: self.config,
            strict_mode: self.strict_mode,
            target: self.target,
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

    #[test]
    #[cfg_attr(miri, ignore)]
    fn disable() {
        let s = Silencer::disable();
        assert_eq!(ULTRASOUND_PERIOD, s.config().intensity());
        assert_eq!(ULTRASOUND_PERIOD, s.config().phase());
        assert!(s.strict_mode());
        assert_eq!(SilencerTarget::Intensity, s.target());
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn from_update_rate() {
        let s = unsafe {
            Silencer::new(FixedUpdateRate {
                intensity: NonZeroU16::new_unchecked(1),
                phase: NonZeroU16::new_unchecked(2),
            })
        };
        assert_eq!(1, s.config().intensity().get());
        assert_eq!(2, s.config().phase().get());
        assert_eq!(SilencerTarget::Intensity, s.target());
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn from_completion_time() {
        let s = Silencer::new(FixedCompletionTime {
            intensity: Duration::from_secs(1),
            phase: Duration::from_secs(1),
        });
        assert_eq!(Duration::from_secs(1), s.config().intensity());
        assert_eq!(Duration::from_secs(1), s.config().phase());
        assert_eq!(SilencerTarget::Intensity, s.target());
    }
}
