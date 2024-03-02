use std::time::Duration;

use crate::{
    common::{LoopBehavior, SamplingConfiguration, Segment},
    defined::float,
    derive::*,
    error::AUTDInternalError,
    operation::ControlPoint,
};

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

#[derive(Debug, Clone, Copy)]
pub struct ChangeFocusSTMSegment {
    segment: Segment,
}

impl ChangeFocusSTMSegment {
    pub const fn new(segment: Segment) -> Self {
        Self { segment }
    }

    pub const fn segment(&self) -> Segment {
        self.segment
    }
}

impl crate::datagram::Datagram for ChangeFocusSTMSegment {
    type O1 = crate::operation::FocusSTMChangeSegmentOp;
    type O2 = crate::operation::NullOp;

    fn timeout(&self) -> Option<Duration> {
        Some(Duration::from_millis(200))
    }

    fn operation(self) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
        Ok((Self::O1::new(self.segment), Self::O2::default()))
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;

    use super::*;
    use crate::{
        geometry::Vector3,
        operation::{FocusSTMOp, NullOp},
    };

    #[test]
    fn new() {
        let stm = FocusSTM::from_freq(1.)
            .add_foci_from_iter((0..10).map(|_| Vector3::zeros()))
            .unwrap();

        assert_eq!(stm.frequency(), 1.);
        assert_eq!(stm.sampling_config().unwrap().frequency(), 1. * 10.);
    }

    #[test]
    fn from_period() {
        let stm = FocusSTM::from_period(std::time::Duration::from_micros(250))
            .add_foci_from_iter((0..10).map(|_| Vector3::zeros()))
            .unwrap();

        assert_eq!(stm.period(), std::time::Duration::from_micros(250));
        assert_eq!(
            stm.sampling_config().unwrap().period(),
            std::time::Duration::from_micros(25)
        );
    }

    #[test]
    fn from_sampling_config() {
        let stm = FocusSTM::from_sampling_config(
            SamplingConfiguration::from_period(std::time::Duration::from_micros(25)).unwrap(),
        )
        .add_foci_from_iter((0..10).map(|_| Vector3::zeros()))
        .unwrap();

        assert_eq!(stm.period(), std::time::Duration::from_micros(250));
        assert_eq!(
            stm.sampling_config().unwrap().period(),
            std::time::Duration::from_micros(25)
        );
    }

    #[test]
    fn add_focus() {
        let stm = FocusSTM::from_freq(1.0)
            .add_focus(Vector3::new(1., 2., 3.))
            .unwrap()
            .add_focus((Vector3::new(4., 5., 6.), 1))
            .unwrap()
            .add_focus(ControlPoint::new(Vector3::new(7., 8., 9.)).with_intensity(2))
            .unwrap();

        assert_eq!(stm.foci().len(), 3);

        assert_eq!(stm[0].point(), &Vector3::new(1., 2., 3.));
        assert_eq!(stm[0].intensity().value(), 0xFF);

        assert_eq!(stm[1].point(), &Vector3::new(4., 5., 6.));
        assert_eq!(stm[1].intensity().value(), 0x01);

        assert_eq!(stm[2].point(), &Vector3::new(7., 8., 9.));
        assert_eq!(stm[2].intensity().value(), 0x02);
    }

    #[test]
    fn add_foci() {
        let stm = FocusSTM::from_freq(1.0)
            .add_foci_from_iter([Vector3::new(1., 2., 3.)])
            .unwrap()
            .add_foci_from_iter([(Vector3::new(4., 5., 6.), 1)])
            .unwrap()
            .add_foci_from_iter([ControlPoint::new(Vector3::new(7., 8., 9.)).with_intensity(2)])
            .unwrap();

        assert_eq!(stm.foci().len(), 3);

        assert_eq!(stm.foci()[0].point(), &Vector3::new(1., 2., 3.));
        assert_eq!(stm.foci()[0].intensity().value(), 0xFF);

        assert_eq!(stm.foci()[1].point(), &Vector3::new(4., 5., 6.));
        assert_eq!(stm.foci()[1].intensity().value(), 0x01);

        assert_eq!(stm.foci()[2].point(), &Vector3::new(7., 8., 9.));
        assert_eq!(stm.foci()[2].intensity().value(), 0x02);
    }

    #[test]
    fn test_with_loop_behavior() {
        let stm = FocusSTM::from_freq(1.0);
        assert_eq!(LoopBehavior::Infinite, stm.loop_behavior());

        let stm = stm.with_loop_behavior(LoopBehavior::Finite(NonZeroU32::new(10).unwrap()));
        assert_eq!(
            LoopBehavior::Finite(NonZeroU32::new(10).unwrap()),
            stm.loop_behavior()
        );
    }

    #[test]
    fn clear() {
        let mut stm = FocusSTM::from_freq(1.0)
            .add_focus(Vector3::new(1., 2., 3.))
            .unwrap()
            .add_focus((Vector3::new(4., 5., 6.), 1))
            .unwrap()
            .add_focus(ControlPoint::new(Vector3::new(7., 8., 9.)).with_intensity(2))
            .unwrap();

        let foci = stm.clear();

        assert_eq!(stm.foci().len(), 0);

        assert_eq!(foci.len(), 3);

        assert_eq!(foci[0].point(), &Vector3::new(1., 2., 3.));
        assert_eq!(foci[0].intensity().value(), 0xFF);
        assert_eq!(foci[1].point(), &Vector3::new(4., 5., 6.));
        assert_eq!(foci[1].intensity().value(), 0x01);
        assert_eq!(foci[2].point(), &Vector3::new(7., 8., 9.));
        assert_eq!(foci[2].intensity().value(), 0x02);
    }

    #[test]
    fn focu_stm_operation() {
        let stm = FocusSTM::from_freq(1.0)
            .add_focus(Vector3::new(1., 2., 3.))
            .unwrap()
            .add_focus((Vector3::new(4., 5., 6.), 1))
            .unwrap()
            .add_focus(ControlPoint::new(Vector3::new(7., 8., 9.)).with_intensity(2))
            .unwrap();

        assert_eq!(stm.timeout(), Some(std::time::Duration::from_millis(200)));

        let r = stm.operation_with_segment(Segment::S0, true);
        assert!(r.is_ok());
        let _: (FocusSTMOp, NullOp) = r.unwrap();
    }

    #[test]
    fn test_change_focus_stm_segment() -> anyhow::Result<()> {
        use crate::datagram::Datagram;
        let d = ChangeFocusSTMSegment::new(Segment::S0);
        assert_eq!(Segment::S0, d.segment());
        assert_eq!(Some(Duration::from_millis(200)), d.timeout());
        let _ = d.operation()?;
        Ok(())
    }
}
