mod implement;
mod mode;

use std::{fmt::Debug, time::Duration};

pub use mode::GainSTMMode;

use super::sampling_config::*;
use crate::{common::Freq, error::AUTDDriverError};

use autd3_core::{
    common::DEFAULT_TIMEOUT,
    datagram::{
        Datagram, DatagramL, DatagramOption, DeviceMask, Inspectable, InspectionResult,
        internal::{HasFiniteLoop, HasSegment},
    },
    environment::Environment,
    firmware::{
        Drive, FirmwareLimits, SamplingConfig, Segment,
        transition_mode::{Ext, GPIO, Immediate, Later, SyncIdx, SysTime, TransitionModeParams},
    },
    gain::{GainCalculator, GainCalculatorGenerator, GainError, TransducerMask},
    geometry::{Device, Geometry},
};

/// A trait to iterate a [`GainCalculator`] for [`GainSTM`].
///
/// [`GainSTM`]: crate::datagram::GainSTM
pub trait GainSTMIterator<'a>: Send + Sync {
    /// The output [`GainCalculator`] type.
    type Calculator: GainCalculator<'a>;

    /// Returns the next [`GainCalculator`].
    fn next(&mut self) -> Option<Self::Calculator>;
}

/// A trait to generate the [`GainSTMIterator`].
pub trait GainSTMIteratorGenerator<'a> {
    /// The element type of the gain sequence.
    type Gain: GainCalculatorGenerator<'a>;
    /// [`GainSTMIterator`] that generates the sequence of [`Gain`].
    ///
    /// [`Gain`]: autd3_core::gain::Gain
    type Iterator: GainSTMIterator<'a, Calculator = <Self::Gain as GainCalculatorGenerator<'a>>::Calculator>;

    /// generates the iterator.
    #[must_use]
    fn generate(&mut self, device: &'a Device) -> Self::Iterator;
}

/// A trait to generate the [`GainSTMIteratorGenerator`].
#[allow(clippy::len_without_is_empty)]
pub trait GainSTMGenerator<'a> {
    /// The type of the iterator generator.
    type T: GainSTMIteratorGenerator<'a>;

    /// Initializes and returns the iterator generator.
    fn init(
        self,
        geometry: &'a Geometry,
        env: &Environment,
        filter: &TransducerMask,
    ) -> Result<Self::T, GainError>;
    /// Returns the length of the sequence of gains.
    #[must_use]
    fn len(&self) -> usize;
}

/// The option for the [`GainSTM`].
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(C)]
pub struct GainSTMOption {
    /// The mode of the STM. The default is [`GainSTMMode::PhaseIntensityFull`].
    pub mode: GainSTMMode,
}

impl Default for GainSTMOption {
    fn default() -> Self {
        Self {
            mode: GainSTMMode::PhaseIntensityFull,
        }
    }
}

/// [`Datagram`] to produce STM by [`Gain`].
///
/// [`Gain`]: autd3_core::gain::Gain
#[derive(Clone, Debug)]
pub struct GainSTM<T, C> {
    /// The sequence of [`Gain`]s.
    ///
    /// [`Gain`]: autd3_core::gain::Gain
    pub gains: T,
    /// The STM configuration.
    pub config: C,
    /// The STM option.
    pub option: GainSTMOption,
}

impl<'a, T: GainSTMGenerator<'a>, C> GainSTM<T, C> {
    /// Create a new [`GainSTM`].
    #[must_use]
    pub const fn new(gains: T, config: C, option: GainSTMOption) -> Self {
        Self {
            gains,
            config,
            option,
        }
    }
}

impl<'a, T: GainSTMGenerator<'a>> GainSTM<T, Freq<f32>> {
    /// Convert to STM with the closest frequency among the possible frequencies.
    #[must_use]
    pub fn into_nearest(self) -> GainSTM<T, FreqNearest> {
        GainSTM {
            gains: self.gains,
            config: FreqNearest(self.config),
            option: self.option,
        }
    }
}

impl<'a, T: GainSTMGenerator<'a>> GainSTM<T, Duration> {
    /// Convert to STM with the closest frequency among the possible period.
    #[must_use]
    pub fn into_nearest(self) -> GainSTM<T, PeriodNearest> {
        GainSTM {
            gains: self.gains,
            config: PeriodNearest(self.config),
            option: self.option,
        }
    }
}

impl<'a, T: GainSTMGenerator<'a>, C: Into<STMConfig> + Copy> GainSTM<T, C> {
    /// The sampling configuration of the STM.
    pub fn sampling_config(&self) -> Result<SamplingConfig, AUTDDriverError> {
        let size = self.gains.len();
        let stm_config: STMConfig = self.config.into();
        stm_config.into_sampling_config(size)
    }
}

pub struct GainSTMOperationGenerator<'a, G> {
    pub(crate) g: G,
    pub(crate) size: usize,
    pub(crate) mode: GainSTMMode,
    pub(crate) sampling_config: SamplingConfig,
    pub(crate) limits: FirmwareLimits,
    pub(crate) rep: u16,
    pub(crate) segment: Segment,
    pub(crate) transition_params: TransitionModeParams,
    pub(crate) __phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a, G: GainSTMGenerator<'a>, C: Into<STMConfig> + std::fmt::Debug> DatagramL<'a>
    for GainSTM<G, C>
{
    type G = GainSTMOperationGenerator<'a, G::T>;
    type Error = AUTDDriverError;

    fn operation_generator_with_finite_loop(
        self,
        geometry: &'a Geometry,
        env: &Environment,
        filter: &DeviceMask,
        limits: &FirmwareLimits,
        segment: Segment,
        transition_params: TransitionModeParams,
        rep: u16,
    ) -> Result<Self::G, Self::Error> {
        let size = self.gains.len();
        let stm_config: STMConfig = self.config.into();
        let sampling_config = stm_config.into_sampling_config(size)?;
        let limits = *limits;
        let GainSTMOption { mode } = self.option;
        let gains = self.gains;
        Ok(GainSTMOperationGenerator {
            g: gains.init(geometry, env, &TransducerMask::from(filter))?,
            size,
            sampling_config,
            limits,
            mode,
            rep,
            segment,
            transition_params,
            __phantom: std::marker::PhantomData,
        })
    }

    fn option(&self) -> DatagramOption {
        DatagramOption {
            timeout: DEFAULT_TIMEOUT,
            parallel_threshold: std::thread::available_parallelism()
                .map(std::num::NonZeroUsize::get)
                .unwrap_or(8),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GainSTMInspectionResult {
    pub name: String,
    pub data: Vec<Vec<Drive>>,
    pub config: SamplingConfig,
    pub mode: GainSTMMode,
}

impl<'a, T: GainSTMGenerator<'a>, C: Into<STMConfig> + Copy + std::fmt::Debug> Inspectable<'a>
    for GainSTM<T, C>
{
    type Result = GainSTMInspectionResult;

    fn inspect(
        self,
        geometry: &'a Geometry,
        env: &Environment,
        filter: &DeviceMask,
        _: &FirmwareLimits,
    ) -> Result<InspectionResult<<Self as Inspectable<'a>>::Result>, <Self as Datagram<'a>>::Error>
    {
        let sampling_config = self.sampling_config()?;
        sampling_config.divide()?;
        let n = self.gains.len();
        let mut g = self
            .gains
            .init(geometry, env, &TransducerMask::from(filter))?;
        let mode = self.option.mode;
        Ok(InspectionResult::new(geometry, filter, |dev| {
            GainSTMInspectionResult {
                name: tynm::type_name::<Self>().to_string(),
                data: {
                    use autd3_core::gain::GainCalculator;
                    let mut d = g.generate(dev);
                    let mut data = Vec::with_capacity(n);
                    while let Some(g) = d.next() {
                        data.push(dev.iter().map(|tr| g.calc(tr)).collect::<Vec<_>>());
                    }
                    data
                },
                config: sampling_config,
                mode,
            }
        }))
    }
}

impl<'a, T: GainSTMGenerator<'a>, C> HasSegment<Immediate> for GainSTM<T, C> {}
impl<'a, T: GainSTMGenerator<'a>, C> HasSegment<Ext> for GainSTM<T, C> {}
impl<'a, T: GainSTMGenerator<'a>, C> HasSegment<Later> for GainSTM<T, C> {}
impl<'a, T: GainSTMGenerator<'a>, C> HasFiniteLoop<SyncIdx> for GainSTM<T, C> {}
impl<'a, T: GainSTMGenerator<'a>, C> HasFiniteLoop<SysTime> for GainSTM<T, C> {}
impl<'a, T: GainSTMGenerator<'a>, C> HasFiniteLoop<GPIO> for GainSTM<T, C> {}
impl<'a, T: GainSTMGenerator<'a>, C> HasFiniteLoop<Later> for GainSTM<T, C> {}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU16;

    use super::*;

    use crate::datagram::{
        GainSTMOption, WithFiniteLoop, WithSegment, gain::tests::TestGain,
        with_loop_behavior::WithFiniteLoopInspectionResult,
        with_segment::WithSegmentInspectionResult,
    };
    use autd3_core::firmware::{Drive, Intensity, Phase, SamplingConfig, Segment, transition_mode};

    #[test]
    fn inspect() -> anyhow::Result<()> {
        let geometry = crate::datagram::gain::tests::create_geometry(2, 1);

        GainSTM {
            gains: [
                TestGain::new(|_dev| |_| Drive::NULL, &geometry),
                TestGain::new(
                    |_dev| {
                        |_| Drive {
                            phase: Phase(0xFF),
                            intensity: Intensity(0xFF),
                        }
                    },
                    &geometry,
                ),
            ],
            config: SamplingConfig::FREQ_4K,
            option: GainSTMOption::default(),
        }
        .inspect(
            &geometry,
            &Environment::default(),
            &DeviceMask::AllEnabled,
            &FirmwareLimits::unused(),
        )?
        .iter()
        .for_each(|r| {
            assert_eq!(
                &Some(GainSTMInspectionResult {
                    name: "GainSTM<[TestGain; 2], SamplingConfig>".to_string(),
                    data: vec![
                        vec![Drive::NULL; 1],
                        vec![
                            Drive {
                                phase: Phase(0xFF),
                                intensity: Intensity(0xFF),
                            };
                            1
                        ],
                    ],
                    config: SamplingConfig::FREQ_4K,
                    mode: GainSTMMode::PhaseIntensityFull,
                }),
                r
            );
        });

        Ok(())
    }

    #[test]
    fn inspect_with_segment() -> anyhow::Result<()> {
        let geometry = crate::datagram::gain::tests::create_geometry(2, 1);

        WithSegment {
            inner: GainSTM {
                gains: vec![
                    TestGain::new(|_dev| |_| Drive::NULL, &geometry),
                    TestGain::new(
                        |_dev| {
                            |_| Drive {
                                phase: Phase(0xFF),
                                intensity: Intensity(0xFF),
                            }
                        },
                        &geometry,
                    ),
                ],
                config: SamplingConfig::FREQ_4K,
                option: GainSTMOption::default(),
            },
            segment: Segment::S1,
            transition_mode: Later,
        }
        .inspect(
            &geometry,
            &Environment::default(),
            &DeviceMask::AllEnabled,
            &FirmwareLimits::unused(),
        )?
        .iter()
        .for_each(|r| {
            assert_eq!(
                &Some(WithSegmentInspectionResult {
                    inner: GainSTMInspectionResult {
                        name: "GainSTM<Vec<TestGain>, SamplingConfig>".to_string(),
                        data: vec![
                            vec![Drive::NULL; 1],
                            vec![
                                Drive {
                                    phase: Phase(0xFF),
                                    intensity: Intensity(0xFF),
                                };
                                1
                            ],
                        ],
                        config: SamplingConfig::FREQ_4K,
                        mode: GainSTMMode::PhaseIntensityFull,
                    },
                    segment: Segment::S1,
                    transition_mode: Later,
                }),
                r
            );
        });

        Ok(())
    }

    #[test]
    fn inspect_with_loop_behavior() -> anyhow::Result<()> {
        let geometry = crate::datagram::gain::tests::create_geometry(2, 1);

        WithFiniteLoop {
            inner: GainSTM {
                gains: vec![
                    TestGain::new(|_dev| |_| Drive::NULL, &geometry),
                    TestGain::new(
                        |_dev| {
                            |_| Drive {
                                phase: Phase(0xFF),
                                intensity: Intensity(0xFF),
                            }
                        },
                        &geometry,
                    ),
                ],
                config: SamplingConfig::FREQ_4K,
                option: GainSTMOption::default(),
            },
            segment: Segment::S1,
            transition_mode: transition_mode::SyncIdx,
            loop_count: NonZeroU16::MIN,
        }
        .inspect(
            &geometry,
            &Environment::default(),
            &DeviceMask::AllEnabled,
            &FirmwareLimits::unused(),
        )?
        .iter()
        .for_each(|r| {
            assert_eq!(
                &Some(WithFiniteLoopInspectionResult {
                    inner: GainSTMInspectionResult {
                        name: "GainSTM<Vec<TestGain>, SamplingConfig>".to_string(),
                        data: vec![
                            vec![Drive::NULL; 1],
                            vec![
                                Drive {
                                    phase: Phase(0xFF),
                                    intensity: Intensity(0xFF),
                                };
                                1
                            ],
                        ],
                        config: SamplingConfig::FREQ_4K,
                        mode: GainSTMMode::PhaseIntensityFull,
                    },
                    segment: Segment::S1,
                    transition_mode: transition_mode::SyncIdx,
                    loop_count: NonZeroU16::MIN,
                }),
                r
            );
        });

        Ok(())
    }
}
