mod implement;
mod mode;

use std::{fmt::Debug, time::Duration};

pub use mode::GainSTMMode;

use super::sampling_config::*;
use crate::{
    common::Freq,
    datagram::{InspectionResultWithLoopBehavior, InspectionResultWithSegment},
    error::AUTDDriverError,
};

use autd3_core::{
    common::DEFAULT_TIMEOUT,
    datagram::{
        Datagram, DatagramL, DatagramOption, DeviceFilter, Inspectable, InspectionResult,
        LoopBehavior, Segment, TransitionMode,
    },
    derive::FirmwareLimits,
    environment::Environment,
    gain::{Drive, GainCalculator, GainCalculatorGenerator, GainError, TransducerFilter},
    geometry::{Device, Geometry},
    sampling_config::SamplingConfig,
};
use derive_more::{Deref, DerefMut};

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
pub trait GainSTMGenerator<'a>: std::fmt::Debug {
    /// The type of the iterator generator.
    type T: GainSTMIteratorGenerator<'a>;

    /// Initializes and returns the iterator generator.
    fn init(
        self,
        geometry: &'a Geometry,
        env: &Environment,
        filter: &TransducerFilter,
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
#[derive(Clone, Debug, Deref, DerefMut)]
pub struct GainSTM<T, C> {
    #[deref]
    #[deref_mut]
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

pub struct GainSTMOperationGenerator<'a, T> {
    pub(crate) g: T,
    pub(crate) size: usize,
    pub(crate) mode: GainSTMMode,
    pub(crate) sampling_config: SamplingConfig,
    pub(crate) limits: FirmwareLimits,
    pub(crate) loop_behavior: LoopBehavior,
    pub(crate) segment: Segment,
    pub(crate) transition_mode: Option<TransitionMode>,
    pub(crate) __phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a, T: GainSTMGenerator<'a>, C: Into<STMConfig> + std::fmt::Debug> DatagramL<'a>
    for GainSTM<T, C>
{
    type G = GainSTMOperationGenerator<'a, T::T>;
    type Error = AUTDDriverError;

    fn operation_generator_with_loop_behavior(
        self,
        geometry: &'a Geometry,
        env: &Environment,
        filter: &DeviceFilter,
        limits: &FirmwareLimits,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
        loop_behavior: LoopBehavior,
    ) -> Result<Self::G, Self::Error> {
        let size = self.gains.len();
        let stm_config: STMConfig = self.config.into();
        let sampling_config = stm_config.into_sampling_config(size)?;
        let limits = *limits;
        let GainSTMOption { mode } = self.option;
        let gains = self.gains;
        Ok(GainSTMOperationGenerator {
            g: gains.init(geometry, env, &TransducerFilter::from(filter))?,
            size,
            sampling_config,
            limits,
            mode,
            loop_behavior,
            segment,
            transition_mode,
            __phantom: std::marker::PhantomData,
        })
    }

    fn option(&self) -> DatagramOption {
        DatagramOption {
            timeout: DEFAULT_TIMEOUT,
            parallel_threshold: num_cpus::get(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GainSTMInspectionResult {
    pub name: String,
    pub data: Vec<Vec<Drive>>,
    pub config: SamplingConfig,
    pub mode: GainSTMMode,
    pub loop_behavior: LoopBehavior,
    pub segment: Segment,
    pub transition_mode: Option<TransitionMode>,
}

impl InspectionResultWithSegment for GainSTMInspectionResult {
    fn with_segment(self, segment: Segment, transition_mode: Option<TransitionMode>) -> Self {
        Self {
            segment,
            transition_mode,
            ..self
        }
    }
}

impl InspectionResultWithLoopBehavior for GainSTMInspectionResult {
    fn with_loop_behavior(
        self,
        loop_behavior: LoopBehavior,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> Self {
        Self {
            loop_behavior,
            segment,
            transition_mode,
            ..self
        }
    }
}

impl<'a, T: GainSTMGenerator<'a>, C: Into<STMConfig> + Copy + std::fmt::Debug> Inspectable<'a>
    for GainSTM<T, C>
{
    type Result = GainSTMInspectionResult;

    fn inspect(
        self,
        geometry: &'a Geometry,
        env: &Environment,
        filter: &DeviceFilter,
        _: &FirmwareLimits,
    ) -> Result<InspectionResult<<Self as Inspectable<'a>>::Result>, <Self as Datagram<'a>>::Error>
    {
        let sampling_config = self.sampling_config()?;
        sampling_config.divide()?;
        let n = self.gains.len();
        let mut g = self
            .gains
            .init(geometry, env, &TransducerFilter::from(filter))?;
        let mode = self.option.mode;
        let loop_behavior = LoopBehavior::Infinite;
        let segment = Segment::S0;
        let transition_mode = None;
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
                loop_behavior,
                segment,
                transition_mode,
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::datagram::{GainSTMOption, WithLoopBehavior, WithSegment, gain::tests::TestGain};
    use autd3_core::gain::{Drive, Intensity, Phase};

    #[test]
    fn inspect() -> anyhow::Result<()> {
        let geometry = crate::datagram::gain::tests::create_geometry(2, 1);

        GainSTM {
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
        }
        .inspect(
            &geometry,
            &Environment::default(),
            &DeviceFilter::all_enabled(),
            &FirmwareLimits::unused(),
        )?
        .iter()
        .for_each(|r| {
            assert_eq!(
                &Some(GainSTMInspectionResult {
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
                    loop_behavior: LoopBehavior::Infinite,
                    segment: Segment::S0,
                    transition_mode: None
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
            transition_mode: Some(TransitionMode::Immediate),
        }
        .inspect(
            &geometry,
            &Environment::default(),
            &DeviceFilter::all_enabled(),
            &FirmwareLimits::unused(),
        )?
        .iter()
        .for_each(|r| {
            assert_eq!(
                &Some(GainSTMInspectionResult {
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
                    loop_behavior: LoopBehavior::Infinite,
                    segment: Segment::S1,
                    transition_mode: Some(TransitionMode::Immediate),
                }),
                r
            );
        });

        Ok(())
    }

    #[test]
    fn inspect_with_loop_behavior() -> anyhow::Result<()> {
        let geometry = crate::datagram::gain::tests::create_geometry(2, 1);

        WithLoopBehavior {
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
            transition_mode: Some(TransitionMode::Immediate),
            loop_behavior: LoopBehavior::ONCE,
        }
        .inspect(
            &geometry,
            &Environment::default(),
            &DeviceFilter::all_enabled(),
            &FirmwareLimits::unused(),
        )?
        .iter()
        .for_each(|r| {
            assert_eq!(
                &Some(GainSTMInspectionResult {
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
                    loop_behavior: LoopBehavior::ONCE,
                    segment: Segment::S1,
                    transition_mode: Some(TransitionMode::Immediate),
                }),
                r
            );
        });

        Ok(())
    }
}
