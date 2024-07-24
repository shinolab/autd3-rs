use std::sync::Arc;

use crate::datagram::*;
use crate::defined::ControlPoints;
use crate::{
    defined::Freq,
    derive::*,
    firmware::{fpga::STMConfig, operation::FociSTMOp},
};

use derive_more::{Deref, DerefMut};

#[derive(Clone, Builder, Deref, DerefMut)]
pub struct FociSTM<const N: usize> {
    #[deref]
    #[deref_mut]
    control_points: Vec<ControlPoints<N>>,
    #[get]
    #[set]
    loop_behavior: LoopBehavior,
    #[get]
    sampling_config: SamplingConfig,
}

impl<const N: usize> FociSTM<N> {
    pub fn new<C, F: IntoIterator<Item = C>>(
        config: impl Into<STMConfig>,
        control_points: F,
    ) -> Result<Self, AUTDInternalError>
    where
        ControlPoints<N>: From<C>,
    {
        let control_points: Vec<_> = control_points
            .into_iter()
            .map(ControlPoints::from)
            .collect();
        if control_points.is_empty() {
            return Err(AUTDInternalError::FociSTMPointSizeOutOfRange(
                control_points.len(),
            ));
        }
        Ok(Self {
            sampling_config: config.into().sampling(control_points.len())?,
            control_points,
            loop_behavior: LoopBehavior::infinite(),
        })
    }

    pub fn freq(&self) -> Result<Freq<f32>, AUTDInternalError> {
        self.sampling_config()
            .freq()
            .map(|f| f / self.control_points.len() as f32)
    }

    pub fn period(&self) -> Result<Duration, AUTDInternalError> {
        self.sampling_config()
            .period()
            .map(|p| p * self.control_points.len() as u32)
    }
}

pub struct FociSTMOperationGenerator<const N: usize> {
    g: Arc<Vec<ControlPoints<N>>>,
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
    segment: Segment,
    transition_mode: Option<TransitionMode>,
}

impl<const N: usize> OperationGenerator for FociSTMOperationGenerator<N> {
    type O1 = FociSTMOp<N>;
    type O2 = NullOp;

    fn generate(&self, _: &Device) -> (Self::O1, Self::O2) {
        (
            Self::O1::new(
                self.g.clone(),
                self.config,
                self.loop_behavior,
                self.segment,
                self.transition_mode,
            ),
            Self::O2::default(),
        )
    }
}

impl<const N: usize> DatagramST for FociSTM<N> {
    type G = FociSTMOperationGenerator<N>;

    fn operation_generator_with_segment(
        self,
        _geometry: &Geometry,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> Result<Self::G, AUTDInternalError> {
        Ok(FociSTMOperationGenerator {
            g: Arc::new(self.control_points),
            config: self.sampling_config,
            loop_behavior: self.loop_behavior,
            segment,
            transition_mode,
        })
    }

    fn timeout(&self) -> Option<std::time::Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn parallel_threshold(&self) -> Option<usize> {
        if self.control_points.len() > 4000 {
            None
        } else {
            Some(usize::MAX)
        }
    }

    #[tracing::instrument(level = "debug", skip(self, _geometry), fields(%self.loop_behavior, %self.sampling_config))]
    // GRCOV_EXCL_START
    fn trace(&self, _geometry: &Geometry) {
        tracing::debug!("{}", tynm::type_name::<Self>());
        match self.control_points.len() {
            0 => {
                tracing::error!("ControlPoints is empty");
            }
            1 => {
                tracing::debug!("ControlPoints: {}", self.control_points[0]);
            }
            2 => {
                tracing::debug!(
                    "ControlPoints: {}, {}",
                    self.control_points[0],
                    self.control_points[1]
                );
            }
            _ => {
                if tracing::enabled!(tracing::Level::TRACE) {
                    tracing::debug!("ControlPoints: {}", self.control_points.iter().join(", "));
                } else {
                    tracing::debug!(
                        "ControlPoints: {}, ..., {} ({})",
                        self.control_points[0],
                        self.control_points[self.control_points.len() - 1],
                        self.control_points.len()
                    );
                }
            }
        }
    }
    // GRCOV_EXCL_STOP
}

#[cfg(feature = "capi")]
impl<const N: usize> Default for FociSTM<N> {
    fn default() -> Self {
        Self {
            control_points: vec![],
            loop_behavior: LoopBehavior::infinite(),
            sampling_config: SamplingConfig::FREQ_40K,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        defined::{kHz, Hz},
        geometry::Vector3,
    };

    use super::*;

    #[rstest::rstest]
    #[test]
    #[case(Ok(SamplingConfig::Freq(1*Hz)), 0.5*Hz, 2)]
    #[case(Ok(SamplingConfig::Freq(10*Hz)), 1.*Hz, 10)]
    #[case(Ok(SamplingConfig::Freq(20*Hz)), 2.*Hz, 10)]
    #[case(Err(AUTDInternalError::STMFreqInvalid(2, 0.49*Hz)), 0.49*Hz, 2)]
    #[case(Err(AUTDInternalError::FociSTMPointSizeOutOfRange(0)), 1.*Hz, 0)]
    #[cfg_attr(miri, ignore)]
    fn from_freq(
        #[case] expect: Result<SamplingConfig, AUTDInternalError>,
        #[case] freq: Freq<f32>,
        #[case] n: usize,
    ) {
        assert_eq!(
            expect,
            FociSTM::new(freq, (0..n).map(|_| Vector3::zeros())).map(|f| f.sampling_config())
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(Ok(SamplingConfig::FreqNearest(1.*Hz)), 0.5*Hz, 2)]
    #[case(Ok(SamplingConfig::FreqNearest(0.98*Hz)), 0.49*Hz, 2)]
    #[case(Ok(SamplingConfig::FreqNearest(10.*Hz)), 1.*Hz, 10)]
    #[case(Ok(SamplingConfig::FreqNearest(20.*Hz)), 2.*Hz, 10)]
    #[cfg_attr(miri, ignore)]
    fn from_freq_nearest(
        #[case] expect: Result<SamplingConfig, AUTDInternalError>,
        #[case] freq: Freq<f32>,
        #[case] n: usize,
    ) {
        assert_eq!(
            expect,
            FociSTM::new(
                STMConfig::FreqNearest(freq),
                (0..n).map(|_| Vector3::zeros())
            )
            .map(|f| f.sampling_config())
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(
        Ok(SamplingConfig::Period(Duration::from_millis(1000))),
        Duration::from_millis(2000),
        2
    )]
    #[case(
        Ok(SamplingConfig::Period(Duration::from_millis(100))),
        Duration::from_millis(1000),
        10
    )]
    #[case(
        Ok(SamplingConfig::Period(Duration::from_millis(50))),
        Duration::from_millis(500),
        10
    )]
    #[case(Err(AUTDInternalError::STMPeriodInvalid(2, Duration::from_millis(2000) + Duration::from_nanos(1))), Duration::from_millis(2000) + Duration::from_nanos(1), 2)]
    #[cfg_attr(miri, ignore)]
    fn from_period(
        #[case] expect: Result<SamplingConfig, AUTDInternalError>,
        #[case] p: Duration,
        #[case] n: usize,
    ) {
        assert_eq!(
            expect,
            FociSTM::new(p, (0..n).map(|_| Vector3::zeros())).map(|f| f.sampling_config())
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(
        Ok(SamplingConfig::PeriodNearest(Duration::from_millis(1000))),
        Duration::from_millis(2000),
        2
    )]
    #[case(
        Ok(SamplingConfig::PeriodNearest(Duration::from_millis(100))),
        Duration::from_millis(1000),
        10
    )]
    #[case(
        Ok(SamplingConfig::PeriodNearest(Duration::from_millis(50))),
        Duration::from_millis(500),
        10
    )]
    #[case(Ok(SamplingConfig::PeriodNearest(Duration::from_millis(1000))), Duration::from_millis(2000) + Duration::from_nanos(1), 2)]
    #[cfg_attr(miri, ignore)]
    fn from_period_nearest(
        #[case] expect: Result<SamplingConfig, AUTDInternalError>,
        #[case] p: Duration,
        #[case] n: usize,
    ) {
        assert_eq!(
            expect,
            FociSTM::new(
                STMConfig::PeriodNearest(p),
                (0..n).map(|_| Vector3::zeros())
            )
            .map(|f| f.sampling_config())
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(SamplingConfig::Freq(4 * kHz), 10)]
    #[case(SamplingConfig::Freq(8 * kHz), 10)]
    #[cfg_attr(miri, ignore)]
    fn from_sampling_config(#[case] config: SamplingConfig, #[case] n: usize) {
        assert_eq!(
            Ok(config),
            FociSTM::new(config, (0..n).map(|_| Vector3::zeros())).map(|f| f.sampling_config())
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(Ok(0.5*Hz), 0.5*Hz, 2)]
    #[case(Ok(1.0*Hz), 1.*Hz, 10)]
    #[case(Ok(2.0*Hz), 2.*Hz, 10)]
    #[case(Err(AUTDInternalError::STMFreqInvalid(2, 0.49*Hz)), 0.49*Hz, 2)]
    #[cfg_attr(miri, ignore)]
    fn freq(
        #[case] expect: Result<Freq<f32>, AUTDInternalError>,
        #[case] f: Freq<f32>,
        #[case] n: usize,
    ) {
        assert_eq!(
            expect,
            FociSTM::new(f, (0..n).map(|_| Vector3::zeros())).and_then(|f| f.freq())
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(Ok(Duration::from_millis(2000)), 0.5*Hz, 2)]
    #[case(Ok(Duration::from_millis(1000)), 1.*Hz, 10)]
    #[case(Ok(Duration::from_millis(500)), 2.*Hz, 10)]
    #[case(Err(AUTDInternalError::STMFreqInvalid(2, 0.49*Hz)), 0.49*Hz, 2)]
    #[cfg_attr(miri, ignore)]
    fn period(
        #[case] expect: Result<Duration, AUTDInternalError>,
        #[case] f: Freq<f32>,
        #[case] n: usize,
    ) {
        assert_eq!(
            expect,
            FociSTM::new(f, (0..n).map(|_| Vector3::zeros())).and_then(|f| f.period())
        );
    }

    #[rstest::rstest]
    #[test]
    #[case::infinite(LoopBehavior::infinite())]
    #[case::finite(LoopBehavior::once())]
    #[cfg_attr(miri, ignore)]
    fn with_loop_behavior(#[case] loop_behavior: LoopBehavior) -> anyhow::Result<()> {
        assert_eq!(
            loop_behavior,
            FociSTM::new(1. * Hz, (0..2).map(|_| Vector3::zeros()))?
                .with_loop_behavior(loop_behavior)
                .loop_behavior()
        );
        Ok(())
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn with_loop_behavior_deafault() -> anyhow::Result<()> {
        let stm = FociSTM::new(1. * Hz, (0..2).map(|_| Vector3::zeros()))?;
        assert_eq!(LoopBehavior::infinite(), stm.loop_behavior());
        Ok(())
    }
}
