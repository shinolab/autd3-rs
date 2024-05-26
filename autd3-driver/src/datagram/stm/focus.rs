use crate::{
    defined::{Freq, DEFAULT_TIMEOUT},
    derive::*,
    firmware::fpga::{STMSamplingConfig, TransitionMode},
};

use super::ControlPoint;

#[derive(Clone, Builder)]
pub struct FocusSTM {
    control_points: Vec<ControlPoint>,
    #[getset]
    loop_behavior: LoopBehavior,
    #[get]
    stm_sampling_config: STMSamplingConfig,
}

impl FocusSTM {
    pub const fn from_freq(freq: Freq<f64>) -> Self {
        Self {
            control_points: Vec::new(),
            loop_behavior: LoopBehavior::infinite(),
            stm_sampling_config: STMSamplingConfig::Freq(freq),
        }
    }

    pub const fn from_freq_nearest(freq: Freq<f64>) -> Self {
        Self {
            control_points: Vec::new(),
            loop_behavior: LoopBehavior::infinite(),
            stm_sampling_config: STMSamplingConfig::FreqNearest(freq),
        }
    }

    pub const fn from_sampling_config(config: SamplingConfig) -> Self {
        Self {
            control_points: Vec::new(),
            loop_behavior: LoopBehavior::infinite(),
            stm_sampling_config: STMSamplingConfig::SamplingConfig(config),
        }
    }

    pub fn add_focus(mut self, point: impl Into<ControlPoint>) -> Self {
        self.control_points.push(point.into());
        self
    }

    pub fn add_foci_from_iter(
        mut self,
        iter: impl IntoIterator<Item = impl Into<ControlPoint>>,
    ) -> Self {
        self.control_points
            .extend(iter.into_iter().map(|c| c.into()));
        self
    }

    pub fn clear(&mut self) -> Vec<ControlPoint> {
        std::mem::take(&mut self.control_points)
    }

    pub fn sampling_config(&self) -> Result<SamplingConfig, AUTDInternalError> {
        self.stm_sampling_config.sampling(self.control_points.len())
    }
}

impl std::ops::Deref for FocusSTM {
    type Target = [ControlPoint];

    fn deref(&self) -> &Self::Target {
        &self.control_points
    }
}

impl std::ops::DerefMut for FocusSTM {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.control_points
    }
}

impl DatagramST for FocusSTM {
    type O1 = crate::firmware::operation::FocusSTMOp;
    type O2 = crate::firmware::operation::NullOp;

    fn operation_with_segment(
        self,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> (Self::O1, Self::O2) {
        let Self {
            control_points,
            loop_behavior,
            stm_sampling_config,
        } = self;
        (
            Self::O1::new(
                control_points,
                stm_sampling_config,
                loop_behavior,
                segment,
                transition_mode,
            ),
            Self::O2::default(),
        )
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
        firmware::operation::FocusSTMOp,
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
            FocusSTM::from_freq(freq)
                .add_foci_from_iter((0..n).map(|_| Vector3::zeros()))
                .sampling_config()
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
            FocusSTM::from_freq_nearest(freq)
                .add_foci_from_iter((0..n).map(|_| Vector3::zeros()))
                .sampling_config()
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(SamplingConfig::DISABLE, 2)]
    #[case(SamplingConfig::Freq(4 * kHz), 10)]
    fn from_sampling_config(
        #[case] config: SamplingConfig,
        #[case] n: usize,
    ) -> anyhow::Result<()> {
        assert_eq!(
            config,
            FocusSTM::from_sampling_config(config)
                .add_foci_from_iter((0..n).map(|_| Vector3::zeros()))
                .sampling_config()?
        );
        Ok(())
    }

    #[test]
    fn add_focus() -> anyhow::Result<()> {
        let stm = FocusSTM::from_freq_nearest(1. * Hz)
            .add_focus(Vector3::new(1., 2., 3.))
            .add_focus((Vector3::new(4., 5., 6.), 1))
            .add_focus(ControlPoint::new(Vector3::new(7., 8., 9.)).with_intensity(2));
        assert_eq!(stm.len(), 3);
        assert_eq!(
            stm[0],
            ControlPoint::new(Vector3::new(1., 2., 3.)).with_intensity(0xFF)
        );
        assert_eq!(
            stm[1],
            ControlPoint::new(Vector3::new(4., 5., 6.)).with_intensity(0x01)
        );
        assert_eq!(
            stm[2],
            ControlPoint::new(Vector3::new(7., 8., 9.)).with_intensity(0x02)
        );
        Ok(())
    }

    #[test]
    fn add_foci() {
        let stm = FocusSTM::from_freq_nearest(1. * Hz)
            .add_foci_from_iter([Vector3::new(1., 2., 3.)])
            .add_foci_from_iter([(Vector3::new(4., 5., 6.), 1)])
            .add_foci_from_iter([ControlPoint::new(Vector3::new(7., 8., 9.)).with_intensity(2)]);
        assert_eq!(stm.len(), 3);
        assert_eq!(
            stm[0],
            ControlPoint::new(Vector3::new(1., 2., 3.)).with_intensity(0xFF)
        );
        assert_eq!(
            stm[1],
            ControlPoint::new(Vector3::new(4., 5., 6.)).with_intensity(0x01)
        );
        assert_eq!(
            stm[2],
            ControlPoint::new(Vector3::new(7., 8., 9.)).with_intensity(0x02)
        );
    }

    #[rstest::rstest]
    #[test]
    #[case::infinite(LoopBehavior::infinite())]
    #[case::finite(LoopBehavior::once())]
    fn with_loop_behavior(#[case] loop_behavior: LoopBehavior) {
        assert_eq!(
            loop_behavior,
            FocusSTM::from_freq(1. * Hz)
                .with_loop_behavior(loop_behavior)
                .loop_behavior()
        );
    }

    #[test]
    fn with_loop_behavior_deafault() {
        let stm = FocusSTM::from_freq(1. * Hz);
        assert_eq!(LoopBehavior::infinite(), stm.loop_behavior());
    }

    #[test]
    fn clear() {
        let mut stm = FocusSTM::from_freq_nearest(1. * Hz)
            .add_focus(Vector3::new(1., 2., 3.))
            .add_focus((Vector3::new(4., 5., 6.), 1))
            .add_focus(ControlPoint::new(Vector3::new(7., 8., 9.)).with_intensity(2));
        let foci = stm.clear();
        assert_eq!(stm.len(), 0);
        assert_eq!(foci.len(), 3);
        assert_eq!(
            foci[0],
            ControlPoint::new(Vector3::new(1., 2., 3.)).with_intensity(0xFF)
        );
        assert_eq!(
            foci[1],
            ControlPoint::new(Vector3::new(4., 5., 6.)).with_intensity(0x01)
        );
        assert_eq!(
            foci[2],
            ControlPoint::new(Vector3::new(7., 8., 9.)).with_intensity(0x02)
        );
    }

    #[test]
    fn deref_mut() {
        let mut stm = FocusSTM::from_freq_nearest(1. * Hz).add_focus(Vector3::new(1., 2., 3.));
        assert_eq!(
            stm[0],
            ControlPoint::new(Vector3::new(1., 2., 3.)).with_intensity(0xFF)
        );
        stm[0] = ControlPoint::new(Vector3::new(4., 5., 6.)).with_intensity(0x01);
        assert_eq!(
            stm[0],
            ControlPoint::new(Vector3::new(4., 5., 6.)).with_intensity(0x01)
        );
    }

    #[test]
    fn operation() {
        let stm = FocusSTM::from_freq_nearest(1. * Hz)
            .add_focus(Vector3::new(1., 2., 3.))
            .add_focus((Vector3::new(4., 5., 6.), 1))
            .add_focus(ControlPoint::new(Vector3::new(7., 8., 9.)).with_intensity(2));

        assert_eq!(Datagram::timeout(&stm), Some(DEFAULT_TIMEOUT));

        let _: (FocusSTMOp, NullOp) =
            stm.operation_with_segment(Segment::S0, Some(TransitionMode::SyncIdx));
    }
}
