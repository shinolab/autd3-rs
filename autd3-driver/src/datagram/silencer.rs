use std::{convert::Infallible, num::NonZeroU16};

use crate::{
    firmware::{
        fpga::{SILENCER_STEPS_INTENSITY_DEFAULT, SILENCER_STEPS_PHASE_DEFAULT},
        operation::{
            NullOp, OperationGenerator, SilencerFixedCompletionStepsOp, SilencerFixedUpdateRateOp,
        },
    },
    geometry::{Device, Geometry},
};

use super::Datagram;

pub trait SilencerConfig: std::fmt::Debug + Clone + Copy {}
impl SilencerConfig for () {}

/// To configure the silencer by the completion time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixedCompletionTime {
    /// The completion time of the intensity change. The value must be multiple of the ultrasound period.
    ///
    /// The larger this value, the more the noise is suppressed.
    pub intensity: std::time::Duration,
    /// The completion time of the phase change. The value must be multiple of the ultrasound period.
    ///
    /// The larger this value, the more the noise is suppressed.
    pub phase: std::time::Duration,
    /// Whether the strict mode is enabled. The default is `true`.
    ///
    /// If the strict mode is enabled, an error is returned if the phase/intensity change of [`Modulation`], [`FociSTM`] or [`GainSTM`] cannot be completed within the time specified by the silencer.
    ///
    /// [`Modulation`]: autd3_core::modulation::Modulation
    /// [`FociSTM`]: crate::datagram::FociSTM
    /// [`GainSTM`]: crate::datagram::GainSTM
    pub strict_mode: bool,
}
impl SilencerConfig for FixedCompletionTime {}

impl Default for FixedCompletionTime {
    fn default() -> Self {
        FixedCompletionTime {
            intensity: SILENCER_STEPS_INTENSITY_DEFAULT as u32
                * autd3_core::defined::ULTRASOUND_PERIOD,
            phase: SILENCER_STEPS_PHASE_DEFAULT as u32 * autd3_core::defined::ULTRASOUND_PERIOD,
            strict_mode: true,
        }
    }
}

/// To configure the silencer by the completion steps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct FixedCompletionSteps {
    /// The completion steps of the intensity change.
    ///
    /// The larger this value, the more the noise is suppressed.
    pub intensity: NonZeroU16,
    /// The completion time of the phase change.
    ///
    /// The larger this value, the more the noise is suppressed.
    pub phase: NonZeroU16,
    /// Whether the strict mode is enabled. The default is `true`.
    ///
    /// If the strict mode is enabled, an error is returned if the phase/intensity change of [`Modulation`], [`FociSTM`] or [`GainSTM`] cannot be completed within the time specified by the silencer.
    ///
    /// [`Modulation`]: autd3_core::modulation::Modulation
    /// [`FociSTM`]: crate::datagram::FociSTM
    /// [`GainSTM`]: crate::datagram::GainSTM
    pub strict_mode: bool,
}
impl SilencerConfig for FixedCompletionSteps {}

impl Default for FixedCompletionSteps {
    fn default() -> Self {
        FixedCompletionSteps {
            intensity: NonZeroU16::new(SILENCER_STEPS_INTENSITY_DEFAULT).unwrap(),
            phase: NonZeroU16::new(SILENCER_STEPS_PHASE_DEFAULT).unwrap(),
            strict_mode: true,
        }
    }
}

/// To configure the silencer by the update rate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct FixedUpdateRate {
    /// The update rate of the intensity change.
    ///
    /// The smaller this value, the more the noise is suppressed.
    pub intensity: NonZeroU16,
    /// The update rate of the phase change.
    ///
    /// The smaller this value, the more the noise is suppressed.
    pub phase: NonZeroU16,
}
impl SilencerConfig for FixedUpdateRate {}

/// [`Datagram`] to configure the silencer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Silencer<T: SilencerConfig> {
    /// Configuration of the silencer.
    pub config: T,
}

impl<T: SilencerConfig> Silencer<T> {
    /// Creates a new [`Silencer`].
    #[must_use]
    pub const fn new(config: T) -> Self {
        Self { config }
    }
}

impl Silencer<()> {
    /// Creates a [`Silencer`] to disable the silencer.
    #[must_use]
    pub const fn disable() -> Silencer<FixedCompletionSteps> {
        Silencer {
            config: FixedCompletionSteps {
                intensity: NonZeroU16::MIN,
                phase: NonZeroU16::MIN,
                strict_mode: true,
            },
        }
    }
}

impl Default for Silencer<FixedCompletionSteps> {
    fn default() -> Self {
        Silencer {
            config: Default::default(),
        }
    }
}

pub struct SilencerOpGenerator<T: SilencerConfig> {
    config: T,
}

impl OperationGenerator for SilencerOpGenerator<FixedUpdateRate> {
    type O1 = SilencerFixedUpdateRateOp;
    type O2 = NullOp;

    fn generate(&mut self, _: &Device) -> Option<(Self::O1, Self::O2)> {
        Some((
            Self::O1::new(self.config.intensity, self.config.phase),
            Self::O2 {},
        ))
    }
}

impl OperationGenerator for SilencerOpGenerator<FixedCompletionTime> {
    type O1 = crate::firmware::operation::SilencerFixedCompletionTimeOp;
    type O2 = NullOp;

    fn generate(&mut self, _: &Device) -> Option<(Self::O1, Self::O2)> {
        Some((
            Self::O1::new(
                self.config.intensity,
                self.config.phase,
                self.config.strict_mode,
            ),
            Self::O2 {},
        ))
    }
}

impl OperationGenerator for SilencerOpGenerator<FixedCompletionSteps> {
    type O1 = SilencerFixedCompletionStepsOp;
    type O2 = NullOp;

    fn generate(&mut self, _: &Device) -> Option<(Self::O1, Self::O2)> {
        Some((
            Self::O1::new(
                self.config.intensity,
                self.config.phase,
                self.config.strict_mode,
            ),
            Self::O2 {},
        ))
    }
}

impl<T: SilencerConfig> Datagram for Silencer<T>
where
    SilencerOpGenerator<T>: OperationGenerator,
{
    type G = SilencerOpGenerator<T>;
    type Error = Infallible;

    fn operation_generator(self, _: &mut Geometry) -> Result<Self::G, Self::Error> {
        Ok(Self::G {
            config: self.config,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disable() {
        let s = Silencer::disable();
        assert_eq!(1, s.config.intensity.get());
        assert_eq!(1, s.config.phase.get());
        assert!(s.config.strict_mode);
    }

    #[test]
    fn fixed_completion_steps_default() {
        let s: Silencer<FixedCompletionSteps> = Silencer::default();
        assert_eq!(10, s.config.intensity.get());
        assert_eq!(40, s.config.phase.get());
        assert!(s.config.strict_mode);
    }

    #[test]
    fn fixed_completion_time_default() {
        let s: Silencer<FixedCompletionTime> = Silencer::new(Default::default());
        assert_eq!(std::time::Duration::from_micros(250), s.config.intensity);
        assert_eq!(std::time::Duration::from_micros(1000), s.config.phase);
        assert!(s.config.strict_mode);
    }
}
