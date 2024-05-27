use crate::{
    datagram::OperationGenerator,
    defined::{ControlPoint, Freq, DEFAULT_TIMEOUT},
    derive::*,
    firmware::{
        fpga::{STMSamplingConfig, TransitionMode},
        operation::FocusSTMOp,
    },
};

#[derive(Clone, Builder)]
pub struct FocusSTM {
    control_points: Vec<ControlPoint>,
    #[getset]
    loop_behavior: LoopBehavior,
    #[get]
    sampling_config: SamplingConfig,
}

impl FocusSTM {
    pub fn from_freq<C, F: ExactSizeIterator<Item = C>>(
        freq: Freq<f64>,
        control_points: F,
    ) -> Result<Self, AUTDInternalError>
    where
        ControlPoint: From<C>,
    {
        let control_points: Vec<_> = control_points.into_iter().map(ControlPoint::from).collect();
        Ok(Self {
            sampling_config: STMSamplingConfig::Freq(freq).sampling(control_points.len())?,
            loop_behavior: LoopBehavior::infinite(),
            control_points,
        })
    }

    pub fn from_freq_nearest<C, F: ExactSizeIterator<Item = C>>(
        freq: Freq<f64>,
        control_points: F,
    ) -> Result<Self, AUTDInternalError>
    where
        ControlPoint: From<C>,
    {
        let control_points: Vec<_> = control_points.into_iter().map(ControlPoint::from).collect();
        Ok(Self {
            sampling_config: STMSamplingConfig::FreqNearest(freq).sampling(control_points.len())?,
            loop_behavior: LoopBehavior::infinite(),
            control_points,
        })
    }

    pub fn from_sampling_config<C, F: ExactSizeIterator<Item = C>>(
        config: SamplingConfig,
        control_points: F,
    ) -> Self
    where
        ControlPoint: From<C>,
    {
        Self {
            control_points: control_points.into_iter().map(ControlPoint::from).collect(),
            loop_behavior: LoopBehavior::infinite(),
            sampling_config: config,
        }
    }
}

pub struct FocusSTMOperationGenerator {
    g: Vec<ControlPoint>,
    config: SamplingConfig,
    rep: u32,
    segment: Segment,
    transition_mode: Option<TransitionMode>,
}

impl<'a> OperationGenerator<'a> for FocusSTMOperationGenerator {
    type O1 = FocusSTMOp;
    type O2 = NullOp;

    fn generate(&'a self, _: &'a Device) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
        Ok((
            Self::O1::new(
                self.g.clone(),
                self.config,
                self.rep,
                self.segment,
                self.transition_mode,
            ),
            Self::O2::default(),
        ))
    }
}

impl<'a> DatagramST<'a> for FocusSTM {
    type O1 = crate::firmware::operation::FocusSTMOp;
    type O2 = crate::firmware::operation::NullOp;
    type G =  FocusSTMOperationGenerator;

    fn operation_generator_with_segment(
        self,
        _geometry: &Geometry,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> Result<Self::G, AUTDInternalError> {
        Ok(FocusSTMOperationGenerator {
            g: self.control_points,
            config: self.sampling_config,
            rep: self.loop_behavior.rep,
            segment,
            transition_mode,
        })
    }

    fn timeout(&self) -> Option<std::time::Duration> {
        Some(DEFAULT_TIMEOUT)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{
        defined::{kHz, Hz},
        geometry::Vector3,
    };

    #[rstest::rstest]
    #[test]
    #[case(Ok(SamplingConfig::Freq(1*Hz)), 0.5*Hz, 2)]
    #[case(Ok(SamplingConfig::Freq(10*Hz)), 1.*Hz, 10)]
    #[case(Ok(SamplingConfig::Freq(20*Hz)), 2.*Hz, 10)]
    #[case(Err(AUTDInternalError::STMFreqInvalid(2, 0.49*Hz)), 0.49*Hz, 2)]
    fn from_freq(
        #[case] expect: Result<SamplingConfig, AUTDInternalError>,
        #[case] freq: Freq<f64>,
        #[case] n: usize,
    ) {
        assert_eq!(
            expect,
            FocusSTM::from_freq(freq, (0..n).map(|_| Vector3::zeros()))
                .map(|f| f.sampling_config())
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
        #[case] freq: Freq<f64>,
        #[case] n: usize,
    ) {
        assert_eq!(
            expect,
            FocusSTM::from_freq_nearest(freq, (0..n).map(|_| Vector3::zeros()))
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
            FocusSTM::from_sampling_config(config, (0..n).map(|_| Vector3::zeros()))
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
            FocusSTM::from_freq(1. * Hz, (0..2).map(|_| Vector3::zeros()))?
                .with_loop_behavior(loop_behavior)
                .loop_behavior()
        );
        Ok(())
    }

    #[test]
    fn with_loop_behavior_deafault() -> anyhow::Result<()> {
        let stm = FocusSTM::from_freq(1. * Hz, (0..2).map(|_| Vector3::zeros()))?;
        assert_eq!(LoopBehavior::infinite(), stm.loop_behavior());
        Ok(())
    }
}
