mod control_point;
mod implement;

pub use control_point::{ControlPoint, ControlPoints};

use std::{fmt::Debug, time::Duration};

use super::sampling_config::*;
use crate::{
    common::Freq,
    error::AUTDDriverError,
    geometry::{Device, Geometry},
};

use autd3_core::{
    common::DEFAULT_TIMEOUT,
    datagram::{
        Datagram, DatagramL, DatagramOption, DeviceMask, Inspectable, InspectionResult,
        internal::{HasFiniteLoop, HasSegment},
    },
    environment::Environment,
    firmware::{
        SamplingConfig, Segment,
        transition_mode::{Ext, GPIO, Immediate, Later, SyncIdx, SysTime, TransitionModeParams},
    },
    geometry::Isometry3,
};

/// A trait to generate a [`ControlPoints`] for  [`FociSTM`].
///
/// [`FociSTM`]: crate::datagram::FociSTM
pub trait FociSTMIterator<const N: usize>: Send {
    /// Returns the next [`ControlPoints`].
    fn next(&mut self, iso: &Isometry3) -> ControlPoints<N>;
}

/// A trait to generate the [`FociSTMIterator`].
pub trait FociSTMIteratorGenerator<const N: usize> {
    /// [`FociSTMIterator`] that generates the sequence of foci.
    type Iterator: FociSTMIterator<N>;

    /// generates the iterator.
    #[must_use]
    fn generate(&mut self, device: &Device) -> Self::Iterator;
}

/// A trait to generate the [`FociSTMIteratorGenerator`].
#[allow(clippy::len_without_is_empty)]
pub trait FociSTMGenerator<const N: usize> {
    /// The type of the iterator generator.
    type T: FociSTMIteratorGenerator<N>;

    /// Initializes and returns the iterator generator.
    fn init(self) -> Result<Self::T, AUTDDriverError>;

    /// Returns the length of the sequence of foci.
    #[must_use]
    fn len(&self) -> usize;
}

/// [`Datagram`] to produce STM by foci.
#[derive(Clone, Debug, PartialEq)]
pub struct FociSTM<const N: usize, T: FociSTMGenerator<N>, C> {
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
    pub(crate) generator: G,
    pub(crate) size: usize,
    pub(crate) config: SamplingConfig,
    pub(crate) sound_speed: f32,
    pub(crate) rep: u16,
    pub(crate) segment: Segment,
    pub(crate) transition_params: TransitionModeParams,
}

impl<const N: usize, G: FociSTMGenerator<N> + std::fmt::Debug, C: Into<STMConfig> + std::fmt::Debug>
    DatagramL<'_> for FociSTM<N, G, C>
{
    type G = FociSTMOperationGenerator<N, G::T>;
    type Error = AUTDDriverError;

    fn operation_generator_with_finite_loop(
        self,
        _: &Geometry,
        env: &Environment,
        _: &DeviceMask,
        segment: Segment,
        transition_params: TransitionModeParams,
        rep: u16,
    ) -> Result<Self::G, Self::Error> {
        let size = self.foci.len();
        let stm_config: STMConfig = self.config.into();
        let sampling_config = stm_config.into_sampling_config(size)?;
        Ok(FociSTMOperationGenerator {
            generator: self.foci.init()?,
            size,
            config: sampling_config,
            sound_speed: env.sound_speed,
            rep,
            segment,
            transition_params,
        })
    }

    fn option(&self) -> DatagramOption {
        DatagramOption {
            parallel_threshold: if self.foci.len() * N >= 4000 {
                std::thread::available_parallelism()
                    .map(std::num::NonZeroUsize::get)
                    .unwrap_or(8)
            } else {
                usize::MAX
            },
            timeout: DEFAULT_TIMEOUT,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FociSTMInspectionResult<const N: usize> {
    pub data: Vec<ControlPoints<N>>,
    pub config: SamplingConfig,
}

impl<
    'a,
    const N: usize,
    G: FociSTMGenerator<N> + std::fmt::Debug,
    C: Into<STMConfig> + Copy + Debug,
> Inspectable<'a> for FociSTM<N, G, C>
{
    type Result = FociSTMInspectionResult<N>;

    fn inspect(
        self,
        geometry: &'a Geometry,
        _: &Environment,
        filter: &DeviceMask,
    ) -> Result<InspectionResult<<Self as Inspectable<'a>>::Result>, <Self as Datagram<'a>>::Error>
    {
        let sampling_config = self.sampling_config()?;
        sampling_config.divide()?;
        let n = self.foci.len();
        let mut g = self.foci.init()?;
        Ok(InspectionResult::new(geometry, filter, |dev| {
            FociSTMInspectionResult {
                data: {
                    let mut d = g.generate(dev);
                    (0..n).map(|_| d.next(dev.inv())).collect()
                },
                config: sampling_config,
            }
        }))
    }
}

impl<const N: usize, T: FociSTMGenerator<N>, C> HasSegment<Immediate> for FociSTM<N, T, C> {}
impl<const N: usize, T: FociSTMGenerator<N>, C> HasSegment<Ext> for FociSTM<N, T, C> {}
impl<const N: usize, T: FociSTMGenerator<N>, C> HasSegment<Later> for FociSTM<N, T, C> {}
impl<const N: usize, T: FociSTMGenerator<N>, C> HasFiniteLoop<SyncIdx> for FociSTM<N, T, C> {}
impl<const N: usize, T: FociSTMGenerator<N>, C> HasFiniteLoop<SysTime> for FociSTM<N, T, C> {}
impl<const N: usize, T: FociSTMGenerator<N>, C> HasFiniteLoop<GPIO> for FociSTM<N, T, C> {}
impl<const N: usize, T: FociSTMGenerator<N>, C> HasFiniteLoop<Later> for FociSTM<N, T, C> {}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU16;

    use crate::datagram::{
        WithFiniteLoop, WithSegment, with_loop_behavior::WithFiniteLoopInspectionResult,
        with_segment::WithSegmentInspectionResult,
    };

    use super::*;

    use autd3_core::{
        firmware::{
            Intensity, Phase, SamplingConfig, Segment,
            transition_mode::{self, Later},
        },
        geometry::Point3,
    };

    #[test]
    fn inspect() -> Result<(), Box<dyn std::error::Error>> {
        let geometry = crate::datagram::gain::tests::create_geometry(2, 1);

        FociSTM {
            foci: [
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
        .inspect(&geometry, &Environment::default(), &DeviceMask::AllEnabled)?
        .iter()
        .for_each(|r| {
            assert_eq!(
                &Some(FociSTMInspectionResult {
                    data: vec![
                        ControlPoints {
                            points: [ControlPoint {
                                point: Point3::origin(),
                                phase_offset: Phase::ZERO,
                            }],
                            intensity: Intensity::MAX
                        },
                        ControlPoints {
                            points: [ControlPoint {
                                point: Point3::new(1., 2., 3.),
                                phase_offset: Phase::PI,
                            }],
                            intensity: Intensity::MAX
                        },
                    ],
                    config: SamplingConfig::FREQ_4K,
                }),
                r
            );
        });

        Ok(())
    }

    #[test]
    fn inspect_with_segment() -> Result<(), Box<dyn std::error::Error>> {
        let geometry = crate::datagram::gain::tests::create_geometry(2, 1);

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
            transition_mode: Later,
        }
        .inspect(&geometry, &Environment::default(), &DeviceMask::AllEnabled)?
        .iter()
        .for_each(|r| {
            assert_eq!(
                &Some(WithSegmentInspectionResult {
                    inner: FociSTMInspectionResult {
                        data: vec![
                            ControlPoints {
                                points: [ControlPoint {
                                    point: Point3::origin(),
                                    phase_offset: Phase::ZERO,
                                }],
                                intensity: Intensity::MAX
                            },
                            ControlPoints {
                                points: [ControlPoint {
                                    point: Point3::new(1., 2., 3.),
                                    phase_offset: Phase::PI,
                                }],
                                intensity: Intensity::MAX
                            },
                        ],
                        config: SamplingConfig::FREQ_4K,
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
    fn inspect_with_loop_behavior() -> Result<(), Box<dyn std::error::Error>> {
        let geometry = crate::datagram::gain::tests::create_geometry(2, 1);

        WithFiniteLoop {
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
            transition_mode: transition_mode::SyncIdx,
            loop_count: NonZeroU16::MIN,
        }
        .inspect(&geometry, &Environment::default(), &DeviceMask::AllEnabled)?
        .iter()
        .for_each(|r| {
            assert_eq!(
                &Some(WithFiniteLoopInspectionResult {
                    inner: FociSTMInspectionResult {
                        data: vec![
                            ControlPoints {
                                points: [ControlPoint {
                                    point: Point3::origin(),
                                    phase_offset: Phase::ZERO,
                                }],
                                intensity: Intensity::MAX
                            },
                            ControlPoints {
                                points: [ControlPoint {
                                    point: Point3::new(1., 2., 3.),
                                    phase_offset: Phase::PI,
                                }],
                                intensity: Intensity::MAX
                            },
                        ],
                        config: SamplingConfig::FREQ_4K,
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
