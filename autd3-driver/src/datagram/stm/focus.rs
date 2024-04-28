use crate::{
    defined::DEFAULT_TIMEOUT,
    derive::*,
    firmware::{
        fpga::{TransitionMode, FOCUS_STM_BUF_SIZE_MAX, STM_BUF_SIZE_MIN},
        operation::ControlPoint,
    },
};

use super::STMProps;

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
    /// * `period` - Period.
    ///
    pub const fn from_period(period: std::time::Duration) -> Self {
        Self::from_props(STMProps::from_period(period))
    }

    /// constructor
    ///
    /// # Arguments
    ///
    /// * `period` - Period. The period closest to `period` from the possible periods is set.
    ///
    pub const fn from_period_nearest(period: std::time::Duration) -> Self {
        Self::from_props(STMProps::from_period_nearest(period))
    }

    /// constructor
    ///
    /// # Arguments
    ///
    /// * `config` - Sampling configuration
    ///
    pub const fn from_sampling_config(config: SamplingConfiguration) -> Self {
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

    pub fn freq(&self) -> Result<f64, AUTDInternalError> {
        self.sampling_config()
            .map(|c| c.freq() / self.control_points.len() as f64)
    }

    pub fn period(&self) -> Result<std::time::Duration, AUTDInternalError> {
        self.sampling_config()
            .map(|c| c.period() * self.control_points.len() as u32)
    }

    pub fn sampling_config(&self) -> Result<SamplingConfiguration, AUTDInternalError> {
        if !(STM_BUF_SIZE_MIN..=FOCUS_STM_BUF_SIZE_MAX).contains(&self.control_points.len()) {
            return Err(AUTDInternalError::FocusSTMPointSizeOutOfRange(
                self.control_points.len(),
            ));
        }
        self.props.sampling_config(self.control_points.len())
    }
}

impl std::ops::Index<usize> for FocusSTM {
    type Output = ControlPoint;

    fn index(&self, idx: usize) -> &Self::Output {
        &self.control_points[idx]
    }
}

impl DatagramS for FocusSTM {
    type O1 = crate::firmware::operation::FocusSTMOp;
    type O2 = crate::firmware::operation::NullOp;

    fn operation_with_segment(
        self,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
        let freq_div = self.sampling_config()?.division();
        let loop_behavior = self.loop_behavior();
        Ok((
            Self::O1::new(
                self.control_points,
                freq_div,
                loop_behavior,
                segment,
                transition_mode,
            ),
            Self::O2::default(),
        ))
    }

    fn timeout(&self) -> Option<std::time::Duration> {
        Some(DEFAULT_TIMEOUT)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::{
        firmware::{fpga::sampling_config, operation::FocusSTMOp},
        geometry::Vector3,
    };

    #[rstest::rstest]
    #[test]
    #[case(Ok(SamplingConfiguration::from_freq(1).unwrap()), 0.5, 2)]
    #[case(Err(AUTDInternalError::STMFrequencyInvalid(2, 0.49, 20000.0)), 0.49, 2)]
    #[case(Ok(SamplingConfiguration::from_freq(10).unwrap()), 1., 10)]
    #[case(Ok(SamplingConfiguration::from_freq(20).unwrap()), 2., 10)]
    #[case(Err(AUTDInternalError::FocusSTMPointSizeOutOfRange(STM_BUF_SIZE_MIN - 1)), 1., STM_BUF_SIZE_MIN - 1)]
    #[case(
        Err(AUTDInternalError::FocusSTMPointSizeOutOfRange(FOCUS_STM_BUF_SIZE_MAX + 1)),
        1.,
        FOCUS_STM_BUF_SIZE_MAX + 1
    )]
    fn from_freq(
        #[case] expect: Result<SamplingConfiguration, AUTDInternalError>,
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
    #[case(Ok(SamplingConfiguration::from_freq_nearest(1.).unwrap()), 0.5, 2)]
    #[case(Ok(SamplingConfiguration::from_freq_nearest(0.98).unwrap()), 0.49, 2)]
    #[case(Ok(SamplingConfiguration::from_freq_nearest(10.).unwrap()), 1., 10)]
    #[case(Ok(SamplingConfiguration::from_freq_nearest(20.).unwrap()), 2., 10)]
    #[case(Err(AUTDInternalError::FocusSTMPointSizeOutOfRange(STM_BUF_SIZE_MIN - 1)), 1., STM_BUF_SIZE_MIN - 1)]
    #[case(
        Err(AUTDInternalError::FocusSTMPointSizeOutOfRange(FOCUS_STM_BUF_SIZE_MAX + 1)),
        1.,
        FOCUS_STM_BUF_SIZE_MAX + 1
    )]
    fn from_freq_nearest(
        #[case] expect: Result<SamplingConfiguration, AUTDInternalError>,
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
    #[case(Ok(SamplingConfiguration::from_period(Duration::from_micros(25)).unwrap()), Duration::from_micros(50), 2)]
    #[case(
        Err(AUTDInternalError::SamplingPeriodInvalid(
            Duration::from_micros(26),
            sampling_config::period_min()
        )),
        Duration::from_micros(52),
        2
    )]
    #[case(Ok(SamplingConfiguration::from_period(Duration::from_micros(25)).unwrap()), Duration::from_micros(250), 10)]
    #[case(Ok(SamplingConfiguration::from_period(Duration::from_micros(50)).unwrap()), Duration::from_micros(500), 10)]
    fn from_period(
        #[case] expect: Result<SamplingConfiguration, AUTDInternalError>,
        #[case] period: Duration,
        #[case] n: usize,
    ) {
        assert_eq!(
            expect,
            FocusSTM::from_period(period)
                .add_foci_from_iter((0..n).map(|_| Vector3::zeros()))
                .sampling_config()
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(Ok(SamplingConfiguration::from_period_nearest(Duration::from_micros(25)).unwrap()), Duration::from_micros(50), 2)]
    #[case(Ok(SamplingConfiguration::from_period_nearest(Duration::from_micros(26)).unwrap()), Duration::from_micros(52), 2)]
    #[case(Ok(SamplingConfiguration::from_period_nearest(Duration::from_micros(25)).unwrap()), Duration::from_micros(250), 10)]
    #[case(Ok(SamplingConfiguration::from_period_nearest(Duration::from_micros(50)).unwrap()), Duration::from_micros(500), 10)]
    fn from_period_nearest(
        #[case] expect: Result<SamplingConfiguration, AUTDInternalError>,
        #[case] period: Duration,
        #[case] n: usize,
    ) {
        assert_eq!(
            expect,
            FocusSTM::from_period_nearest(period)
                .add_foci_from_iter((0..n).map(|_| Vector3::zeros()))
                .sampling_config()
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(SamplingConfiguration::DISABLE, 2)]
    #[case(SamplingConfiguration::FREQ_4K_HZ, 10)]
    fn from_sampling_config(
        #[case] config: SamplingConfiguration,
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

    #[rstest::rstest]
    #[test]
    #[case(Ok(1.0), FocusSTM::from_freq(1.0), 2)]
    #[case(Ok(1.0), FocusSTM::from_freq(1.0), 10)]
    #[case(Ok(1.0), FocusSTM::from_period(Duration::from_secs(1)), 2)]
    #[case(Ok(1.0), FocusSTM::from_period(Duration::from_secs(1)), 10)]
    #[case(
        Ok(400.0),
        FocusSTM::from_sampling_config(SamplingConfiguration::FREQ_4K_HZ),
        10
    )]
    #[case(Err(AUTDInternalError::FocusSTMPointSizeOutOfRange(STM_BUF_SIZE_MIN - 1)), FocusSTM::from_freq(1.), STM_BUF_SIZE_MIN - 1)]
    fn freq(
        #[case] expect: Result<f64, AUTDInternalError>,
        #[case] stm: FocusSTM,
        #[case] n: usize,
    ) -> anyhow::Result<()> {
        assert_eq!(
            expect,
            stm.add_foci_from_iter((0..n).map(|_| Vector3::zeros()))
                .freq()
        );
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    #[case(Ok(Duration::from_secs(1)), FocusSTM::from_freq(1.0), 2)]
    #[case(Ok(Duration::from_secs(1)), FocusSTM::from_freq(1.0), 10)]
    #[case(
        Ok(Duration::from_secs(1)),
        FocusSTM::from_period(Duration::from_secs(1)),
        2
    )]
    #[case(
        Ok(Duration::from_secs(1)),
        FocusSTM::from_period(Duration::from_secs(1)),
        10
    )]
    #[case(
        Ok(Duration::from_micros(2500)),
        FocusSTM::from_sampling_config(SamplingConfiguration::FREQ_4K_HZ),
        10
    )]
    #[case(Err(AUTDInternalError::FocusSTMPointSizeOutOfRange(STM_BUF_SIZE_MIN - 1)), FocusSTM::from_freq(1.), STM_BUF_SIZE_MIN - 1)]
    fn period(
        #[case] expect: Result<Duration, AUTDInternalError>,
        #[case] stm: FocusSTM,
        #[case] n: usize,
    ) -> anyhow::Result<()> {
        assert_eq!(
            expect,
            stm.add_foci_from_iter((0..n).map(|_| Vector3::zeros()))
                .period()
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
    fn operation() -> anyhow::Result<()> {
        let stm = FocusSTM::from_freq_nearest(1.)
            .add_focus(Vector3::new(1., 2., 3.))
            .add_focus((Vector3::new(4., 5., 6.), 1))
            .add_focus(ControlPoint::new(Vector3::new(7., 8., 9.)).with_intensity(2));

        assert_eq!(stm.timeout(), Some(DEFAULT_TIMEOUT));

        let r = stm.operation_with_segment(Segment::S0, Some(TransitionMode::SyncIdx));
        assert!(r.is_ok());
        let _: (FocusSTMOp, NullOp) = r.unwrap();
        Ok(())
    }
}
