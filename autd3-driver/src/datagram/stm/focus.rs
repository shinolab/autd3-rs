use crate::{derive::*, operation::ControlPoint};

use super::STMProps;

/// FocusSTM is an STM for moving a single focal point.
///
/// The sampling timing is determined by hardware, thus the sampling time is precise.
///
/// FocusSTM has following restrictions:
/// - The maximum number of sampling points is [crate::fpga::FOCUS_STM_BUF_SIZE_MAX].
/// - The sampling frequency is [crate::fpga::FPGA_CLK_FREQ]/N, where `N` is a 32-bit unsigned integer and must be at least [crate::fpga::SAMPLING_FREQ_DIV_MIN]
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
    /// * `freq` - Frequency of STM. The frequency closest to `freq` from the possible frequencies is set.
    ///
    pub const fn from_freq(freq: float) -> Self {
        Self::from_props(STMProps::from_freq(freq))
    }

    /// constructor
    ///
    /// # Arguments
    ///
    /// * `period` - Period. The period closest to `period` from the possible periods is set.
    ///
    pub const fn from_period(period: std::time::Duration) -> Self {
        Self::from_props(STMProps::from_period(period))
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
    pub fn add_focus(mut self, point: impl Into<ControlPoint>) -> Result<Self, AUTDInternalError> {
        self.control_points.push(point.into());
        self.props.sampling_config(self.control_points.len())?;
        Ok(self)
    }

    /// Add [ControlPoint]s to FocusSTM
    pub fn add_foci_from_iter(
        mut self,
        iter: impl IntoIterator<Item = impl Into<ControlPoint>>,
    ) -> Result<Self, AUTDInternalError> {
        self.control_points
            .extend(iter.into_iter().map(|c| c.into()));
        self.props.sampling_config(self.control_points.len())?;
        Ok(self)
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

    pub fn frequency(&self) -> float {
        self.props.freq(self.control_points.len())
    }

    pub fn period(&self) -> std::time::Duration {
        self.props.period(self.control_points.len())
    }

    pub fn sampling_config(&self) -> Result<SamplingConfiguration, AUTDInternalError> {
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
    type O1 = crate::operation::FocusSTMOp;
    type O2 = crate::operation::NullOp;

    fn operation_with_segment(
        self,
        segment: crate::common::Segment,
        update_segment: bool,
    ) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
        let freq_div = self.sampling_config()?.frequency_division();
        let loop_behavior = self.loop_behavior();
        Ok((
            Self::O1::new(
                self.control_points,
                freq_div,
                loop_behavior,
                segment,
                update_segment,
            ),
            Self::O2::default(),
        ))
    }

    fn timeout(&self) -> Option<std::time::Duration> {
        Some(std::time::Duration::from_millis(200))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::{geometry::Vector3, operation::FocusSTMOp};

    #[rstest::rstest]
    #[test]
    #[case(0.5, 2)]
    #[case(1.0, 10)]
    #[case(2.0, 10)]
    fn test_from_requency(#[case] freq: float, #[case] n: usize) -> anyhow::Result<()> {
        let stm = FocusSTM::from_freq(freq).add_foci_from_iter((0..n).map(|_| Vector3::zeros()))?;
        assert_eq!(freq, stm.frequency());
        assert_eq!(freq * n as float, stm.sampling_config()?.frequency());
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    #[case(Duration::from_micros(125), 2)]
    #[case(Duration::from_micros(250), 10)]
    #[case(Duration::from_micros(500), 10)]
    fn from_period(#[case] period: Duration, #[case] n: usize) -> anyhow::Result<()> {
        let stm =
            FocusSTM::from_period(period).add_foci_from_iter((0..n).map(|_| Vector3::zeros()))?;
        assert_eq!(period, stm.period());
        assert_eq!(period / n as u32, stm.sampling_config()?.period());
        Ok(())
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
                .add_foci_from_iter((0..n).map(|_| Vector3::zeros()))?
                .sampling_config()?
        );
        Ok(())
    }

    #[test]
    fn test_add_focus() -> anyhow::Result<()> {
        let stm = FocusSTM::from_freq(1.0)
            .add_focus(Vector3::new(1., 2., 3.))?
            .add_focus((Vector3::new(4., 5., 6.), 1))?
            .add_focus(ControlPoint::new(Vector3::new(7., 8., 9.)).with_intensity(2))?;
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
    fn test_add_foci() -> anyhow::Result<()> {
        let stm = FocusSTM::from_freq(1.0)
            .add_foci_from_iter([Vector3::new(1., 2., 3.)])?
            .add_foci_from_iter([(Vector3::new(4., 5., 6.), 1)])?
            .add_foci_from_iter([ControlPoint::new(Vector3::new(7., 8., 9.)).with_intensity(2)])?;
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
    #[case::infinite(LoopBehavior::Infinite)]
    #[case::finite(LoopBehavior::once())]
    fn test_with_loop_behavior(#[case] loop_behavior: LoopBehavior) {
        assert_eq!(
            loop_behavior,
            FocusSTM::from_freq(1.0)
                .with_loop_behavior(loop_behavior)
                .loop_behavior()
        );
    }

    #[test]
    fn test_with_loop_behavior_deafault() {
        let stm = FocusSTM::from_freq(1.0);
        assert_eq!(LoopBehavior::Infinite, stm.loop_behavior());
    }

    #[test]
    fn test_clear() -> anyhow::Result<()> {
        let mut stm = FocusSTM::from_freq(1.0)
            .add_focus(Vector3::new(1., 2., 3.))?
            .add_focus((Vector3::new(4., 5., 6.), 1))?
            .add_focus(ControlPoint::new(Vector3::new(7., 8., 9.)).with_intensity(2))?;
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
    fn test_operation() -> anyhow::Result<()> {
        let stm = FocusSTM::from_freq(1.0)
            .add_focus(Vector3::new(1., 2., 3.))?
            .add_focus((Vector3::new(4., 5., 6.), 1))?
            .add_focus(ControlPoint::new(Vector3::new(7., 8., 9.)).with_intensity(2))?;

        assert_eq!(stm.timeout(), Some(Duration::from_millis(200)));

        let r = stm.operation_with_segment(Segment::S0, true);
        assert!(r.is_ok());
        let _: (FocusSTMOp, NullOp) = r.unwrap();
        Ok(())
    }
}
