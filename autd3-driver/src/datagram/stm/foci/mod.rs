mod implement;

use std::{fmt::Debug, time::Duration};

use super::sampling_config::*;
use crate::{
    common::Freq,
    datagram::{
        with_loop_behavior::InspectionResultWithLoopBehavior,
        with_segment::InspectionResultWithSegment, *,
    },
    firmware::{
        fpga::{LoopBehavior, SamplingConfig, Segment, TransitionMode},
        operation::FociSTMOp,
    },
};

pub use crate::firmware::operation::FociSTMIterator;

use autd3_core::{
    common::DEFAULT_TIMEOUT,
    datagram::{DatagramL, DatagramOption, Inspectable},
    derive::InspectionResult,
};
use derive_more::{Deref, DerefMut};

/// A trait to generate the [`FociSTMIterator`].
pub trait FociSTMIteratorGenerator<const N: usize>: std::fmt::Debug {
    /// [`FociSTMIterator`] that generates the sequence of foci.
    type Iterator: FociSTMIterator<N>;

    /// generates the iterator.
    #[must_use]
    fn generate(&mut self, device: &Device) -> Self::Iterator;
}

/// A trait to generate the [`FociSTMIteratorGenerator`].
#[allow(clippy::len_without_is_empty)]
pub trait FociSTMGenerator<const N: usize>: std::fmt::Debug {
    /// The type of the iterator generator.
    type T: FociSTMIteratorGenerator<N>;

    /// Initializes and returns the iterator generator.
    fn init(self) -> Result<Self::T, AUTDDriverError>;

    /// Returns the length of the sequence of foci.
    #[must_use]
    fn len(&self) -> usize;
}

/// [`Datagram`] to produce STM by foci.
#[derive(Clone, Deref, DerefMut, Debug, PartialEq)]
pub struct FociSTM<const N: usize, T: FociSTMGenerator<N>, C> {
    #[deref]
    #[deref_mut]
    /// The sequence of foci.
    pub foci: T,
    /// The STM configuration.
    pub config: C,
}

impl<const N: usize, T: FociSTMGenerator<N>, C> FociSTM<N, T, C> {
    /// Create a new FociSTM.
    #[must_use]
    pub const fn new(foci: T, config: C) -> Self {
        Self { foci, config }
    }
}

impl<const N: usize, T: FociSTMGenerator<N>> FociSTM<N, T, Freq<f32>> {
    /// Convert to STM with the closest frequency among the possible frequencies.
    #[must_use]
    pub fn into_nearest(self) -> FociSTM<N, T, FreqNearest> {
        FociSTM {
            foci: self.foci,
            config: FreqNearest(self.config),
        }
    }
}

impl<const N: usize, T: FociSTMGenerator<N>> FociSTM<N, T, Duration> {
    /// Convert to STM with the closest frequency among the possible period.
    #[must_use]
    pub fn into_nearest(self) -> FociSTM<N, T, PeriodNearest> {
        FociSTM {
            foci: self.foci,
            config: PeriodNearest(self.config),
        }
    }
}

impl<const N: usize, T: FociSTMGenerator<N>, C: Into<STMConfig> + Copy> FociSTM<N, T, C> {
    /// The sampling configuration of the STM.
    pub fn sampling_config(&self) -> Result<SamplingConfig, AUTDDriverError> {
        let size = self.foci.len();
        let stm_config: STMConfig = self.config.into();
        stm_config.into_sampling_config(size)
    }
}

pub struct FociSTMOperationGenerator<const N: usize, G: FociSTMIteratorGenerator<N>> {
    generator: G,
    size: usize,
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
    segment: Segment,
    transition_mode: Option<TransitionMode>,
}

impl<const N: usize, G: FociSTMIteratorGenerator<N>> OperationGenerator
    for FociSTMOperationGenerator<N, G>
{
    type O1 = FociSTMOp<N, G::Iterator>;
    type O2 = NullOp;

    fn generate(&mut self, device: &Device) -> Option<(Self::O1, Self::O2)> {
        Some((
            Self::O1::new(
                self.generator.generate(device),
                self.size,
                self.config,
                self.loop_behavior,
                self.segment,
                self.transition_mode,
            ),
            Self::O2 {},
        ))
    }
}

impl<const N: usize, G: FociSTMGenerator<N>, C: Into<STMConfig> + Debug> DatagramL
    for FociSTM<N, G, C>
{
    type G = FociSTMOperationGenerator<N, G::T>;
    type Error = AUTDDriverError;

    fn operation_generator_with_loop_behavior(
        self,
        _: &Geometry,
        _: &DeviceFilter,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
        loop_behavior: LoopBehavior,
    ) -> Result<Self::G, Self::Error> {
        let size = self.foci.len();
        let stm_config: STMConfig = self.config.into();
        let sampling_config = stm_config.into_sampling_config(size)?;
        Ok(FociSTMOperationGenerator {
            generator: self.foci.init()?,
            size,
            config: sampling_config,
            loop_behavior,
            segment,
            transition_mode,
        })
    }

    fn option(&self) -> DatagramOption {
        DatagramOption {
            parallel_threshold: if self.foci.len() * N >= 4000 {
                num_cpus::get()
            } else {
                usize::MAX
            },
            timeout: DEFAULT_TIMEOUT,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FociSTMInspectionResult<const N: usize> {
    pub name: String,
    pub data: Vec<ControlPoints<N>>,
    pub config: SamplingConfig,
    pub loop_behavior: LoopBehavior,
    pub segment: Segment,
    pub transition_mode: Option<TransitionMode>,
}

impl<const N: usize> InspectionResultWithSegment for FociSTMInspectionResult<N> {
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

impl<const N: usize> InspectionResultWithLoopBehavior for FociSTMInspectionResult<N> {
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

impl<const N: usize, G: FociSTMGenerator<N>, C: Into<STMConfig> + Copy + Debug> Inspectable
    for FociSTM<N, G, C>
{
    type Result = FociSTMInspectionResult<N>;

    fn inspect(
        self,
        geometry: &Geometry,
        filter: &DeviceFilter,
    ) -> Result<InspectionResult<<Self as Inspectable>::Result>, <Self as Datagram>::Error> {
        let sampling_config = self.sampling_config()?;
        sampling_config.divide()?;
        let n = self.foci.len();
        let mut g = self.foci.init()?;
        let loop_behavior = LoopBehavior::Infinite;
        let segment = Segment::S0;
        let transition_mode = None;
        Ok(InspectionResult::new(geometry, filter, |dev| {
            FociSTMInspectionResult {
                name: tynm::type_name::<Self>().to_string(),
                data: {
                    let mut d = g.generate(dev);
                    (0..n).map(|_| d.next()).collect()
                },
                config: sampling_config,
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

    use crate::{datagram::tests::create_geometry, firmware::fpga::SamplingConfig};
    use autd3_core::{
        derive::Inspectable,
        gain::{EmitIntensity, Phase},
        geometry::Point3,
    };

    #[test]
    fn inspect() -> anyhow::Result<()> {
        let geometry = create_geometry(2, 1);

        FociSTM {
            foci: vec![
                ControlPoint {
                    point: Point3::origin(),
                    phase_offset: Phase::ZERO,
                },
                ControlPoint {
                    point: Point3::new(1., 2., 3.),
                    phase_offset: Phase::PI,
                },
            ],
            config: SamplingConfig::FREQ_4K,
        }
        .inspect(&geometry, &DeviceFilter::all_enabled())?
        .iter()
        .for_each(|r| {
            assert_eq!(
                &Some(FociSTMInspectionResult {
                    name: "FociSTM".to_string(),
                    data: vec![
                        ControlPoints {
                            points: [ControlPoint {
                                point: Point3::origin(),
                                phase_offset: Phase::ZERO,
                            }],
                            intensity: EmitIntensity::MAX
                        },
                        ControlPoints {
                            points: [ControlPoint {
                                point: Point3::new(1., 2., 3.),
                                phase_offset: Phase::PI,
                            }],
                            intensity: EmitIntensity::MAX
                        },
                    ],
                    config: SamplingConfig::FREQ_4K,
                    loop_behavior: LoopBehavior::Infinite,
                    segment: Segment::S0,
                    transition_mode: None,
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
            inner: FociSTM {
                foci: vec![
                    ControlPoint {
                        point: Point3::origin(),
                        phase_offset: Phase::ZERO,
                    },
                    ControlPoint {
                        point: Point3::new(1., 2., 3.),
                        phase_offset: Phase::PI,
                    },
                ],
                config: SamplingConfig::FREQ_4K,
            },
            segment: Segment::S1,
            transition_mode: Some(TransitionMode::Immediate),
        }
        .inspect(&geometry, &DeviceFilter::all_enabled())?
        .iter()
        .for_each(|r| {
            assert_eq!(
                &Some(FociSTMInspectionResult {
                    name: "FociSTM".to_string(),
                    data: vec![
                        ControlPoints {
                            points: [ControlPoint {
                                point: Point3::origin(),
                                phase_offset: Phase::ZERO,
                            }],
                            intensity: EmitIntensity::MAX
                        },
                        ControlPoints {
                            points: [ControlPoint {
                                point: Point3::new(1., 2., 3.),
                                phase_offset: Phase::PI,
                            }],
                            intensity: EmitIntensity::MAX
                        },
                    ],
                    config: SamplingConfig::FREQ_4K,
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
            inner: FociSTM {
                foci: vec![
                    ControlPoint {
                        point: Point3::origin(),
                        phase_offset: Phase::ZERO,
                    },
                    ControlPoint {
                        point: Point3::new(1., 2., 3.),
                        phase_offset: Phase::PI,
                    },
                ],
                config: SamplingConfig::FREQ_4K,
            },
            segment: Segment::S1,
            transition_mode: Some(TransitionMode::Immediate),
            loop_behavior: LoopBehavior::ONCE,
        }
        .inspect(&geometry, &DeviceFilter::all_enabled())?
        .iter()
        .for_each(|r| {
            assert_eq!(
                &Some(FociSTMInspectionResult {
                    name: "FociSTM".to_string(),
                    data: vec![
                        ControlPoints {
                            points: [ControlPoint {
                                point: Point3::origin(),
                                phase_offset: Phase::ZERO,
                            }],
                            intensity: EmitIntensity::MAX
                        },
                        ControlPoints {
                            points: [ControlPoint {
                                point: Point3::new(1., 2., 3.),
                                phase_offset: Phase::PI,
                            }],
                            intensity: EmitIntensity::MAX
                        },
                    ],
                    config: SamplingConfig::FREQ_4K,
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
