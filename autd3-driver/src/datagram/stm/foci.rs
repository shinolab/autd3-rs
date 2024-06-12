use std::sync::Arc;

use crate::datagram::*;
use crate::defined::ControlPoints;
use crate::{
    defined::Freq,
    derive::*,
    firmware::{fpga::STMSamplingConfig, operation::FociSTMOp},
};

use derive_more::{Deref, DerefMut};

#[derive(Clone, Builder, Deref, DerefMut)]
pub struct FociSTM<const N: usize> {
    #[deref]
    #[deref_mut]
    control_points: Vec<ControlPoints<N>>,
    #[getset]
    loop_behavior: LoopBehavior,
    #[get]
    sampling_config: SamplingConfig,
}

impl<const N: usize> FociSTM<N> {
    pub fn from_freq<C, F: IntoIterator<Item = C>>(
        freq: Freq<f32>,
        control_points: F,
    ) -> Result<Self, AUTDInternalError>
    where
        ControlPoints<N>: From<C>,
    {
        let control_points: Vec<_> = control_points
            .into_iter()
            .map(ControlPoints::from)
            .collect();
        Ok(Self {
            sampling_config: STMSamplingConfig::Freq(freq).sampling(control_points.len())?,
            loop_behavior: LoopBehavior::infinite(),
            control_points,
        })
    }

    pub fn from_freq_nearest<C, F: IntoIterator<Item = C>>(
        freq: Freq<f32>,
        control_points: F,
    ) -> Result<Self, AUTDInternalError>
    where
        ControlPoints<N>: From<C>,
    {
        let control_points: Vec<_> = control_points
            .into_iter()
            .map(ControlPoints::from)
            .collect();
        Ok(Self {
            sampling_config: STMSamplingConfig::FreqNearest(freq).sampling(control_points.len())?,
            loop_behavior: LoopBehavior::infinite(),
            control_points,
        })
    }

    pub fn from_sampling_config<C, F: IntoIterator<Item = C>>(
        config: SamplingConfig,
        control_points: F,
    ) -> Self
    where
        ControlPoints<N>: From<C>,
    {
        Self {
            control_points: control_points
                .into_iter()
                .map(ControlPoints::from)
                .collect(),
            loop_behavior: LoopBehavior::infinite(),
            sampling_config: config,
        }
    }
}

pub struct FociSTMOperationGenerator<const N: usize> {
    g: Arc<Vec<ControlPoints<N>>>,
    config: SamplingConfig,
    rep: u32,
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
                self.rep,
                self.segment,
                self.transition_mode,
            ),
            Self::O2::default(),
        )
    }
}

impl<const N: usize> DatagramST for FociSTM<N> {
    type O1 = FociSTMOp<N>;
    type O2 = NullOp;
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
            rep: self.loop_behavior.rep(),
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
        tracing::info!("{}", tynm::type_name::<Self>());
        if tracing::enabled!(tracing::Level::DEBUG) {
            if tracing::enabled!(tracing::Level::TRACE) {
                self.control_points.iter().enumerate().for_each(|(i, f)| {
                    tracing::debug!("ControlPoints[{}]: {:?}", i, f);
                });
            } else {
                let len = self.control_points.len();
                tracing::debug!("ControlPoints[{}]: {:?}", 0, self.control_points[0]);
                if len > 2 {
                    tracing::debug!("︙");
                }
                if len > 1 {
                    tracing::debug!(
                        "ControlPoints[{}]: {:?}",
                        len - 1,
                        self.control_points[len - 1]
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
            sampling_config: SamplingConfig::DISABLE,
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
    fn from_freq(
        #[case] expect: Result<SamplingConfig, AUTDInternalError>,
        #[case] freq: Freq<f32>,
        #[case] n: usize,
    ) {
        assert_eq!(
            expect,
            FociSTM::from_freq(freq, (0..n).map(|_| Vector3::zeros())).map(|f| f.sampling_config())
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(Ok(SamplingConfig::FreqNearest(1.*Hz)), 0.5*Hz, 2)]
    #[case(Ok(SamplingConfig::FreqNearest(0.98*Hz)), 0.49*Hz, 2)]
    #[case(Ok(SamplingConfig::FreqNearest(10.*Hz)), 1.*Hz, 10)]
    #[case(Ok(SamplingConfig::FreqNearest(20.*Hz)), 2.*Hz, 10)]
    fn from_freq_nearest(
        #[case] expect: Result<SamplingConfig, AUTDInternalError>,
        #[case] freq: Freq<f32>,
        #[case] n: usize,
    ) {
        assert_eq!(
            expect,
            FociSTM::from_freq_nearest(freq, (0..n).map(|_| Vector3::zeros()))
                .map(|f| f.sampling_config())
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(SamplingConfig::DISABLE, 2)]
    #[case(SamplingConfig::Freq(4 * kHz), 10)]
    fn from_sampling_config(#[case] config: SamplingConfig, #[case] n: usize) {
        assert_eq!(
            config,
            FociSTM::from_sampling_config(config, (0..n).map(|_| Vector3::zeros()))
                .sampling_config()
        );
    }

    #[rstest::rstest]
    #[test]
    #[case::infinite(LoopBehavior::infinite())]
    #[case::finite(LoopBehavior::once())]
    fn with_loop_behavior(#[case] loop_behavior: LoopBehavior) -> anyhow::Result<()> {
        assert_eq!(
            loop_behavior,
            FociSTM::from_freq(1. * Hz, (0..2).map(|_| Vector3::zeros()))?
                .with_loop_behavior(loop_behavior)
                .loop_behavior()
        );
        Ok(())
    }

    #[test]
    fn with_loop_behavior_deafault() -> anyhow::Result<()> {
        let stm = FociSTM::from_freq(1. * Hz, (0..2).map(|_| Vector3::zeros()))?;
        assert_eq!(LoopBehavior::infinite(), stm.loop_behavior());
        Ok(())
    }
}
