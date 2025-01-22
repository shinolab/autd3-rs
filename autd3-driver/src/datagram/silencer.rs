use std::{convert::Infallible, num::NonZeroU16};

use autd3_core::datagram::DatagramOption;

use crate::{
    firmware::{
        fpga::{SilencerTarget, SILENCER_STEPS_INTENSITY_DEFAULT, SILENCER_STEPS_PHASE_DEFAULT},
        operation::{
            NullOp, OperationGenerator, SilencerFixedCompletionStepsOp, SilencerFixedUpdateRateOp,
        },
    },
    geometry::{Device, Geometry},
};

use super::Datagram;

pub trait SilencerConfig: std::fmt::Debug + Clone + Copy {}
impl SilencerConfig for () {}

#[cfg(not(feature = "dynamic_freq"))]
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
#[cfg(not(feature = "dynamic_freq"))]
impl SilencerConfig for FixedCompletionTime {}

#[cfg(not(feature = "dynamic_freq"))]
impl Default for FixedCompletionTime {
    fn default() -> Self {
        FixedCompletionTime {
            intensity: SILENCER_STEPS_INTENSITY_DEFAULT as u32
                * autd3_core::defined::ultrasound_period(),
            phase: SILENCER_STEPS_PHASE_DEFAULT as u32 * autd3_core::defined::ultrasound_period(),
            strict_mode: true,
        }
    }
}

/// To configure the silencer by the completion steps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy)]
pub struct Silencer<T: SilencerConfig> {
    /// Configuration of the silencer.
    pub config: T,
    /// The target of the silencer.
    pub target: SilencerTarget,
}

impl Silencer<()> {
    /// Creates a [`Silencer`] to disable the silencer.
    pub const fn disable() -> Silencer<FixedCompletionSteps> {
        Silencer {
            config: FixedCompletionSteps {
                intensity: NonZeroU16::MIN,
                phase: NonZeroU16::MIN,
                strict_mode: true,
            },
            target: SilencerTarget::Intensity,
        }
    }
}

impl Default for Silencer<FixedCompletionSteps> {
    fn default() -> Self {
        Silencer {
            config: Default::default(),
            target: Default::default(),
        }
    }
}

pub struct SilencerOpGenerator<T: SilencerConfig> {
    config: T,
    target: SilencerTarget,
}

impl OperationGenerator for SilencerOpGenerator<FixedUpdateRate> {
    type O1 = SilencerFixedUpdateRateOp;
    type O2 = NullOp;

    fn generate(&mut self, _: &Device) -> (Self::O1, Self::O2) {
        (
            Self::O1::new(self.config.intensity, self.config.phase, self.target),
            Self::O2 {},
        )
    }
}

#[cfg(not(feature = "dynamic_freq"))]
impl OperationGenerator for SilencerOpGenerator<FixedCompletionTime> {
    type O1 = crate::firmware::operation::SilencerFixedCompletionTimeOp;
    type O2 = NullOp;

    fn generate(&mut self, _: &Device) -> (Self::O1, Self::O2) {
        (
            Self::O1::new(
                self.config.intensity,
                self.config.phase,
                self.config.strict_mode,
                self.target,
            ),
            Self::O2 {},
        )
    }
}

impl OperationGenerator for SilencerOpGenerator<FixedCompletionSteps> {
    type O1 = SilencerFixedCompletionStepsOp;
    type O2 = NullOp;

    fn generate(&mut self, _: &Device) -> (Self::O1, Self::O2) {
        (
            Self::O1::new(
                self.config.intensity,
                self.config.phase,
                self.config.strict_mode,
                self.target,
            ),
            Self::O2 {},
        )
    }
}

impl<T: SilencerConfig> Datagram for Silencer<T>
where
    SilencerOpGenerator<T>: OperationGenerator,
{
    type G = SilencerOpGenerator<T>;
    type Error = Infallible;

    fn operation_generator(self, _: &Geometry, _: &DatagramOption) -> Result<Self::G, Self::Error> {
        Ok(Self::G {
            config: self.config,
            target: self.target,
        })
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::{datagram::FociSTM, firmware::fpga::LoopBehavior, geometry::Point3};

//     use super::*;

//     #[test]
//     fn disable() {
//         let s = Silencer::disable();
//         assert_eq!(1, s.config.intensity.get());
//         assert_eq!(1, s.config.phase.get());
//         assert!(s.config.strict_mode);
//         assert_eq!(SilencerTarget::Intensity, s.target);
//     }

//     #[rstest::rstest]
//     #[test]
//     // #[case(true, 10, 10, true, FociSTM::new(SamplingConfig::new(10).unwrap(), [Point3::origin()]).unwrap())]
//     // #[case(false, 11, 10, true, FociSTM::new(SamplingConfig::new(10).unwrap(), [Point3::origin()]).unwrap())]
//     // #[case(false, 10, 11, true, FociSTM::new(SamplingConfig::new(10).unwrap(), [Point3::origin()]).unwrap())]
//     // #[case(true, 11, 10, false, FociSTM::new(SamplingConfig::new(10).unwrap(), [Point3::origin()]).unwrap())]
//     // #[case(true, 10, 11, false, FociSTM::new(SamplingConfig::new(10).unwrap(), [Point3::origin()]).unwrap())]
//     // #[case(true, 10, 10, true, GainSTM::new(SamplingConfig::new(10).unwrap(), [TestGain{ data: Default::default() }]).unwrap())]
//     // #[case(false, 11, 10, true, GainSTM::new(SamplingConfig::new(10).unwrap(), [TestGain{ data: Default::default() }]).unwrap())]
//     // #[case(false, 10, 11, true, GainSTM::new(SamplingConfig::new(10).unwrap(), [TestGain{ data: Default::default() }]).unwrap())]
//     // #[case(true, 11, 10, false, GainSTM::new(SamplingConfig::new(10).unwrap(), [TestGain{ data: Default::default() }]).unwrap())]
//     // #[case(true, 10, 11, false, GainSTM::new(SamplingConfig::new(10).unwrap(), [TestGain{ data: Default::default() }]).unwrap())]
//     // #[case(true, 10, 10, true, TestModulation { config: SamplingConfig::new(10).unwrap(), loop_behavior: LoopBehavior::Infinite })]
//     // #[case(false, 11, 10, true, TestModulation { config: SamplingConfig::new(10).unwrap(), loop_behavior: LoopBehavior::Infinite })]
//     // #[case(true, 10, 11, true, TestModulation { config: SamplingConfig::new(10).unwrap(), loop_behavior: LoopBehavior::Infinite })]
//     // #[case(true, 11, 10, false, TestModulation { config: SamplingConfig::new(10).unwrap(), loop_behavior: LoopBehavior::Infinite })]
//     // #[case(true, 10, 11, false, TestModulation { config: SamplingConfig::new(10).unwrap(), loop_behavior: LoopBehavior::Infinite })]
//     fn fixed_completion_steps_is_valid(
//         #[case] expect: bool,
//         #[case] intensity: u16,
//         #[case] phase: u16,
//         #[case] strict_mode: bool,
//         #[case] target: impl HasSamplingConfig,
//     ) {
//         let s = Silencer {
//             config: FixedCompletionSteps {
//                 intensity: NonZeroU16::new(intensity).unwrap(),
//                 phase: NonZeroU16::new(phase).unwrap(),
//                 strict_mode,
//             },
//             target: SilencerTarget::Intensity,
//         };
//         assert_eq!(expect, s.is_valid(&target));
//     }

//     #[cfg(not(feature = "dynamic_freq"))]
//     #[rstest::rstest]
//     #[test]
//     // #[case(true, 10, 10, true, FociSTM::new(SamplingConfig::new(10).unwrap(), [Point3::origin()]).unwrap())]
//     // #[case(false, 11, 10, true, FociSTM::new(SamplingConfig::new(10).unwrap(), [Point3::origin()]).unwrap())]
//     // #[case(false, 10, 11, true, FociSTM::new(SamplingConfig::new(10).unwrap(), [Point3::origin()]).unwrap())]
//     // #[case(true, 11, 10, false, FociSTM::new(SamplingConfig::new(10).unwrap(), [Point3::origin()]).unwrap())]
//     // #[case(true, 10, 11, false, FociSTM::new(SamplingConfig::new(10).unwrap(), [Point3::origin()]).unwrap())]
//     // #[case(true, 10, 10, true, GainSTM::new(SamplingConfig::new(10).unwrap(), [TestGain{ data: Default::default() }]).unwrap())]
//     // #[case(false, 11, 10, true, GainSTM::new(SamplingConfig::new(10).unwrap(), [TestGain{ data: Default::default() }]).unwrap())]
//     // #[case(false, 10, 11, true, GainSTM::new(SamplingConfig::new(10).unwrap(), [TestGain{ data: Default::default() }]).unwrap())]
//     // #[case(true, 11, 10, false, GainSTM::new(SamplingConfig::new(10).unwrap(), [TestGain{ data: Default::default() }]).unwrap())]
//     // #[case(true, 10, 11, false, GainSTM::new(SamplingConfig::new(10).unwrap(), [TestGain{ data: Default::default() }]).unwrap())]
//     // #[case(true, 10, 10, true, TestModulation { config: SamplingConfig::new(10).unwrap(), loop_behavior: LoopBehavior::Infinite })]
//     // #[case(false, 11, 10, true, TestModulation { config: SamplingConfig::new(10).unwrap(), loop_behavior: LoopBehavior::Infinite })]
//     // #[case(true, 10, 11, true, TestModulation { config: SamplingConfig::new(10).unwrap(), loop_behavior: LoopBehavior::Infinite })]
//     // #[case(true, 11, 10, false, TestModulation { config: SamplingConfig::new(10).unwrap(), loop_behavior: LoopBehavior::Infinite })]
//     // #[case(true, 10, 11, false, TestModulation { config: SamplingConfig::new(10).unwrap(), loop_behavior: LoopBehavior::Infinite })]
//     // fn fixed_completion_time_is_valid(
//     //     #[case] expect: bool,
//     //     #[case] intensity: u32,
//     //     #[case] phase: u32,
//     //     #[case] strict_mode: bool,
//     //     #[case] target: impl HasSamplingConfig,
//     // ) {
//     //     use crate::defined::ultrasound_period;

//     //     let s = Silencer {
//     //         config: FixedCompletionTime {
//     //             intensity: intensity * ultrasound_period(),
//     //             phase: phase * ultrasound_period(),
//     //             strict_mode,
//     //         },
//     //         target: SilencerTarget::Intensity,
//     //     };
//     //     assert_eq!(expect, s.is_valid(&target));
//     // }
// }
