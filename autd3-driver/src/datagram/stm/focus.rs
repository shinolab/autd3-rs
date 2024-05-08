use crate::{defined::DEFAULT_TIMEOUT, derive::*, firmware::fpga::TransitionMode};

use super::{ControlPoint, STMProps};

/// FocusSTM is an STM for moving a single focal point.
///
/// The sampling timing is determined by hardware, thus the sampling time is precise.
///
/// FocusSTM has following restrictions:
/// - The maximum number of sampling points is [crate::fpga::FOCUS_STM_BUF_SIZE_MAX].
/// - The sampling freq is [crate::firmware::fpga::fpga_clk_freq()]/N, where `N` is a 32-bit unsigned integer and must be at least [crate::fpga::SAMPLING_FREQ_DIV_MIN]
///
#[derive(Clone, Builder)]
pub struct FocusSTM {
    control_points: Vec<ControlPoint>,
    #[getset(loop_behavior: LoopBehavior)]
    props: STMProps,
}

impl FocusSTM {
    /// constructor
    ///
    /// # Arguments
    ///
    /// * `freq` - Frequency of STM.
    ///
    pub const fn from_freq(freq: f64) -> Self {
        Self::from_props(STMProps::from_freq(freq))
    }

    /// constructor
    ///
    /// # Arguments
    ///
    /// * `freq` - Frequency of STM. The freq closest to `freq` from the possible frequencies is set.
    ///
    pub const fn from_freq_nearest(freq: f64) -> Self {
        Self::from_props(STMProps::from_freq_nearest(freq))
    }

    /// constructor
    ///
    /// # Arguments
    ///
    /// * `config` - Sampling configuration
    ///
    pub const fn from_sampling_config(config: SamplingConfig) -> Self {
        Self::from_props(STMProps::from_sampling_config(config))
    }

    /// constructor
    ///
    /// # Arguments
    ///
    /// * `props` - STMProps
    pub const fn from_props(props: STMProps) -> Self {
        Self {
            control_points: Vec::new(),
            props,
        }
    }

    /// Add [ControlPoint] to FocusSTM
    pub fn add_focus(mut self, point: impl Into<ControlPoint>) -> Self {
        self.control_points.push(point.into());
        self
    }

    /// Add [ControlPoint]s to FocusSTM
    pub fn add_foci_from_iter(
        mut self,
        iter: impl IntoIterator<Item = impl Into<ControlPoint>>,
    ) -> Self {
        self.control_points
            .extend(iter.into_iter().map(|c| c.into()));
        self
    }

    /// Clear current [ControlPoint]s
    ///
    /// # Returns
    /// removed [ControlPoint]s
    pub fn clear(&mut self) -> Vec<ControlPoint> {
        std::mem::take(&mut self.control_points)
    }

    /// Get [ControlPoint]s
    pub fn foci(&self) -> &[ControlPoint] {
        &self.control_points
    }

    pub fn sampling_config(&self) -> Result<SamplingConfig, AUTDInternalError> {
        self.props.sampling_config(self.control_points.len())
    }
}

impl std::ops::Index<usize> for FocusSTM {
    type Output = ControlPoint;

    fn index(&self, idx: usize) -> &Self::Output {
        &self.control_points[idx]
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
            props: STMProps {
                loop_behavior,
                config,
            },
            ..
        } = self;
        (
            Self::O1::new(
                control_points,
                config,
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
    use crate::{firmware::operation::FocusSTMOp, geometry::Vector3};

    #[rstest::rstest]
    #[test]
    #[case(Ok(SamplingConfig::Freq(1)), 0.5, 2)]
    #[case(Ok(SamplingConfig::Freq(10)), 1., 10)]
    #[case(Ok(SamplingConfig::Freq(20)), 2., 10)]
    #[case(Err(AUTDInternalError::STMFreqInvalid(2, 0.49)), 0.49, 2)]
    fn from_freq(
        #[case] expect: Result<SamplingConfig, AUTDInternalError>,
        #[case] freq: f64,
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
    #[case(Ok(SamplingConfig::FreqNearest(1.)), 0.5, 2)]
    #[case(Ok(SamplingConfig::FreqNearest(0.98)), 0.49, 2)]
    #[case(Ok(SamplingConfig::FreqNearest(10.)), 1., 10)]
    #[case(Ok(SamplingConfig::FreqNearest(20.)), 2., 10)]
    fn from_freq_nearest(
        #[case] expect: Result<SamplingConfig, AUTDInternalError>,
        #[case] freq: f64,
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
    #[case(SamplingConfig::FREQ_4K_HZ, 10)]
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
        let stm = FocusSTM::from_freq_nearest(1.)
            .add_focus(Vector3::new(1., 2., 3.))
            .add_focus((Vector3::new(4., 5., 6.), 1))
            .add_focus(ControlPoint::new(Vector3::new(7., 8., 9.)).with_intensity(2));
        assert_eq!(stm.foci().len(), 3);
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
    fn add_foci() -> anyhow::Result<()> {
        let stm = FocusSTM::from_freq_nearest(1.)
            .add_foci_from_iter([Vector3::new(1., 2., 3.)])
            .add_foci_from_iter([(Vector3::new(4., 5., 6.), 1)])
            .add_foci_from_iter([ControlPoint::new(Vector3::new(7., 8., 9.)).with_intensity(2)]);
        assert_eq!(stm.foci().len(), 3);
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

    #[rstest::rstest]
    #[test]
    #[case::infinite(LoopBehavior::infinite())]
    #[case::finite(LoopBehavior::once())]
    fn with_loop_behavior(#[case] loop_behavior: LoopBehavior) {
        assert_eq!(
            loop_behavior,
            FocusSTM::from_freq(1.)
                .with_loop_behavior(loop_behavior)
                .loop_behavior()
        );
    }

    #[test]
    fn with_loop_behavior_deafault() {
        let stm = FocusSTM::from_freq(1.);
        assert_eq!(LoopBehavior::infinite(), stm.loop_behavior());
    }

    #[test]
    fn clear() -> anyhow::Result<()> {
        let mut stm = FocusSTM::from_freq_nearest(1.)
            .add_focus(Vector3::new(1., 2., 3.))
            .add_focus((Vector3::new(4., 5., 6.), 1))
            .add_focus(ControlPoint::new(Vector3::new(7., 8., 9.)).with_intensity(2));
        let foci = stm.clear();
        assert_eq!(stm.foci().len(), 0);
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
        Ok(())
    }

    #[test]
    fn operation() {
        let stm = FocusSTM::from_freq_nearest(1.)
            .add_focus(Vector3::new(1., 2., 3.))
            .add_focus((Vector3::new(4., 5., 6.), 1))
            .add_focus(ControlPoint::new(Vector3::new(7., 8., 9.)).with_intensity(2));

        assert_eq!(Datagram::timeout(&stm), Some(DEFAULT_TIMEOUT));

        let _: (FocusSTMOp, NullOp) =
            stm.operation_with_segment(Segment::S0, Some(TransitionMode::SyncIdx));
    }
}
