mod implement;

use std::{fmt::Debug, time::Duration};

use super::sampling_config::*;
pub use crate::firmware::operation::GainSTMIterator;
use crate::{
    common::Freq,
    datagram::{
        with_loop_behavior::InspectionResultWithLoopBehavior,
        with_segment::InspectionResultWithSegment, *,
    },
    firmware::{
        cpu::GainSTMMode,
        fpga::{LoopBehavior, SamplingConfig, Segment, TransitionMode},
        operation::GainSTMOp,
    },
};

use autd3_core::{
    common::DEFAULT_TIMEOUT,
    datagram::{DatagramL, DatagramOption, Inspectable, InspectionResult},
    gain::{Drive, GainCalculatorGenerator, GainError, TransducerFilter},
};
use derive_more::{Deref, DerefMut};

/// A trait to generate the [`GainSTMIterator`].
pub trait GainSTMIteratorGenerator {
    /// The element type of the gain sequence.
    type Gain: GainCalculatorGenerator;
    /// [`GainSTMIterator`] that generates the sequence of [`Gain`].
    ///
    /// [`Gain`]: autd3_core::gain::Gain
    type Iterator: GainSTMIterator<Calculator = <Self::Gain as GainCalculatorGenerator>::Calculator>;

    /// generates the iterator.
    #[must_use]
    fn generate(&mut self, device: &Device) -> Self::Iterator;
}

/// A trait to generate the [`GainSTMIteratorGenerator`].
#[allow(clippy::len_without_is_empty)]
pub trait GainSTMGenerator: std::fmt::Debug {
    /// The type of the iterator generator.
    type T: GainSTMIteratorGenerator;

    /// Initializes and returns the iterator generator.
    fn init(self, geometry: &Geometry, filter: &TransducerFilter) -> Result<Self::T, GainError>;
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
pub struct GainSTM<T: GainSTMGenerator, C> {
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

impl<T: GainSTMGenerator, C> GainSTM<T, C> {
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

impl<T: GainSTMGenerator> GainSTM<T, Freq<f32>> {
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

impl<T: GainSTMGenerator> GainSTM<T, Duration> {
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

impl<T: GainSTMGenerator, C: Into<STMConfig> + Copy> GainSTM<T, C> {
    /// The sampling configuration of the STM.
    pub fn sampling_config(&self) -> Result<SamplingConfig, AUTDDriverError> {
        let size = self.gains.len();
        let stm_config: STMConfig = self.config.into();
        stm_config.into_sampling_config(size)
    }
}

pub struct GainSTMOperationGenerator<T: GainSTMIteratorGenerator> {
    g: T,
    size: usize,
    mode: GainSTMMode,
    sampling_config: SamplingConfig,
    loop_behavior: LoopBehavior,
    segment: Segment,
    transition_mode: Option<TransitionMode>,
}

impl<T: GainSTMIteratorGenerator> OperationGenerator for GainSTMOperationGenerator<T> {
    type O1 = GainSTMOp<<T::Gain as GainCalculatorGenerator>::Calculator, T::Iterator>;
    type O2 = NullOp;

    fn generate(&mut self, device: &Device) -> Option<(Self::O1, Self::O2)> {
        Some((
            Self::O1::new(
                self.g.generate(device),
                self.size,
                self.mode,
                self.sampling_config,
                self.loop_behavior,
                self.segment,
                self.transition_mode,
            ),
            Self::O2 {},
        ))
    }
}

impl<T: GainSTMGenerator, C: Into<STMConfig> + Debug> DatagramL for GainSTM<T, C> {
    type G = GainSTMOperationGenerator<T::T>;
    type Error = AUTDDriverError;

    fn operation_generator_with_loop_behavior(
        self,
        geometry: &Geometry,
        filter: &DeviceFilter,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
        loop_behavior: LoopBehavior,
    ) -> Result<Self::G, Self::Error> {
        let size = self.gains.len();
        let stm_config: STMConfig = self.config.into();
        let sampling_config = stm_config.into_sampling_config(size)?;
        let GainSTMOption { mode } = self.option;
        let gains = self.gains;
        Ok(GainSTMOperationGenerator {
            g: gains.init(geometry, &TransducerFilter::from(filter))?,
            size,
            sampling_config,
            mode,
            loop_behavior,
            segment,
            transition_mode,
        })
    }

    fn option(&self) -> DatagramOption {
        DatagramOption {
            parallel_threshold: 4,
            timeout: DEFAULT_TIMEOUT,
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
    fn with_segment(
        self,
        segment: autd3_core::derive::Segment,
        transition_mode: Option<autd3_core::derive::TransitionMode>,
    ) -> Self {
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
        loop_behavior: autd3_core::derive::LoopBehavior,
        segment: autd3_core::derive::Segment,
        transition_mode: Option<autd3_core::derive::TransitionMode>,
    ) -> Self {
        Self {
            loop_behavior,
            segment,
            transition_mode,
            ..self
        }
    }
}

impl<T: GainSTMGenerator, C: Into<STMConfig> + Copy + Debug> Inspectable for GainSTM<T, C> {
    type Result = GainSTMInspectionResult;

    fn inspect(
        self,
        geometry: &Geometry,
        filter: &DeviceFilter,
    ) -> Result<InspectionResult<<Self as Inspectable>::Result>, <Self as Datagram>::Error> {
        let sampling_config = self.sampling_config()?;
        sampling_config.divide()?;
        let n = self.gains.len();
        let mut g = self.gains.init(geometry, &TransducerFilter::from(filter))?;
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

    use crate::{
        datagram::{GainSTMOption, gain::tests::TestGain, tests::create_geometry},
        firmware::fpga::SamplingConfig,
    };
    use autd3_core::{
        derive::Inspectable,
        gain::{Drive, EmitIntensity, Phase},
    };

    #[test]
    fn inspect() -> anyhow::Result<()> {
        let geometry = create_geometry(2, 1);

        GainSTM {
            gains: vec![
                TestGain::new(|_dev| |_| Drive::NULL, &geometry),
                TestGain::new(
                    |_dev| {
                        |_| Drive {
                            phase: Phase(0xFF),
                            intensity: EmitIntensity(0xFF),
                        }
                    },
                    &geometry,
                ),
            ],
            config: SamplingConfig::FREQ_4K,
            option: GainSTMOption::default(),
        }
        .inspect(&geometry, &DeviceFilter::all_enabled())?
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
                                intensity: EmitIntensity(0xFF),
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
        let geometry = create_geometry(2, 1);

        WithSegment {
            inner: GainSTM {
                gains: vec![
                    TestGain::new(|_dev| |_| Drive::NULL, &geometry),
                    TestGain::new(
                        |_dev| {
                            |_| Drive {
                                phase: Phase(0xFF),
                                intensity: EmitIntensity(0xFF),
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
        .inspect(&geometry, &DeviceFilter::all_enabled())?
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
                                intensity: EmitIntensity(0xFF),
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
        let geometry = create_geometry(2, 1);

        WithLoopBehavior {
            inner: GainSTM {
                gains: vec![
                    TestGain::new(|_dev| |_| Drive::NULL, &geometry),
                    TestGain::new(
                        |_dev| {
                            |_| Drive {
                                phase: Phase(0xFF),
                                intensity: EmitIntensity(0xFF),
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
        .inspect(&geometry, &DeviceFilter::all_enabled())?
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
                                intensity: EmitIntensity(0xFF),
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
