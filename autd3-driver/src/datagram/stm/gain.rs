use std::time::Duration;

use crate::{
    common::Segment,
    common::{LoopBehavior, SamplingConfiguration},
    datagram::{DatagramS, Gain},
    defined::float,
    error::AUTDInternalError,
    operation::GainSTMMode,
};

use super::STMProps;

/// GainSTM is an STM for moving [Gain].
///
/// The sampling timing is determined by hardware, thus the sampling time is precise.
///
/// GainSTM has following restrictions:
/// - The maximum number of sampling [Gain] is [crate::fpga::GAIN_STM_BUF_SIZE_MAX].
/// - The sampling frequency is [crate::fpga::FPGA_CLK_FREQ]/N, where `N` is a 32-bit unsigned integer and must be at least [crate::fpga::SAMPLING_FREQ_DIV_MIN]
///
pub struct GainSTM<G: Gain> {
    gains: Vec<G>,
    mode: GainSTMMode,
    props: STMProps,
}

impl<G: Gain> GainSTM<G> {
    /// constructor
    ///
    /// # Arguments
    ///
    /// * `freq` - Frequency of STM. The frequency closest to `freq` from the possible frequencies is set.
    ///
    pub const fn from_freq(freq: float) -> Self {
        Self::from_props_mode(STMProps::from_freq(freq), GainSTMMode::PhaseIntensityFull)
    }

    /// constructor
    ///
    /// # Arguments
    ///
    /// * `period` - Period. The period closest to `period` from the possible periods is set.
    ///
    pub const fn from_period(period: std::time::Duration) -> Self {
        Self::from_props_mode(
            STMProps::from_period(period),
            GainSTMMode::PhaseIntensityFull,
        )
    }

    /// constructor
    ///
    /// # Arguments
    ///
    /// * `freq_div` - Sampling frequency division of STM. The sampling frequency is [crate::fpga::FPGA_CLK_FREQ]/`freq_div`.
    ///
    pub const fn from_sampling_config(config: SamplingConfiguration) -> Self {
        Self::from_props_mode(
            STMProps::from_sampling_config(config),
            GainSTMMode::PhaseIntensityFull,
        )
    }

    /// Set loop behavior
    pub fn with_loop_behavior(self, loop_behavior: LoopBehavior) -> Self {
        Self {
            props: self.props.with_loop_behavior(loop_behavior),
            ..self
        }
    }

    pub const fn loop_behavior(&self) -> LoopBehavior {
        self.props.loop_behavior()
    }

    pub fn frequency(&self) -> float {
        self.props.freq(self.gains.len())
    }

    pub fn period(&self) -> std::time::Duration {
        self.props.period(self.gains.len())
    }

    pub fn sampling_config(&self) -> Result<SamplingConfiguration, AUTDInternalError> {
        self.props.sampling_config(self.gains.len())
    }

    /// Set the mode of GainSTM
    pub fn with_mode(self, mode: GainSTMMode) -> Self {
        Self { mode, ..self }
    }

    pub const fn mode(&self) -> GainSTMMode {
        self.mode
    }

    /// Add a [Gain] to GainSTM
    pub fn add_gain(mut self, gain: G) -> Result<Self, AUTDInternalError> {
        self.gains.push(gain);
        self.props.sampling_config(self.gains.len())?;
        Ok(self)
    }

    /// Add boxed [Gain]s from iterator to GainSTM
    pub fn add_gains_from_iter(
        mut self,
        iter: impl IntoIterator<Item = G>,
    ) -> Result<Self, AUTDInternalError> {
        self.gains.extend(iter);
        self.props.sampling_config(self.gains.len())?;
        Ok(self)
    }

    /// constructor
    ///
    /// # Arguments
    ///
    /// * `props` - STMProps
    /// * `mode` - GainSTMMode
    pub const fn from_props_mode(props: STMProps, mode: GainSTMMode) -> Self {
        Self {
            gains: Vec::new(),
            mode,
            props,
        }
    }

    /// Get [Gain]s
    pub fn gains(&self) -> &[G] {
        &self.gains
    }

    /// Clear current [Gain]s
    ///
    /// # Returns
    /// removed [Gain]s
    pub fn clear(&mut self) -> Vec<G> {
        std::mem::take(&mut self.gains)
    }
}

impl<G: Gain> std::ops::Index<usize> for GainSTM<G> {
    type Output = G;

    fn index(&self, idx: usize) -> &Self::Output {
        &self.gains[idx]
    }
}

impl<G: Gain> DatagramS for GainSTM<G> {
    type O1 = crate::operation::GainSTMOp<G>;
    type O2 = crate::operation::NullOp;

    fn operation_with_segment(
        self,
        segment: crate::common::Segment,
        update_segment: bool,
    ) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
        let freq_div = self.sampling_config()?.frequency_division();
        let Self {
            gains,
            mode,
            props: STMProps { loop_behavior, .. },
            ..
        } = self;
        Ok((
            Self::O1::new(
                gains,
                mode,
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
pub struct ChangeGainSTMSegment {
    segment: Segment,
}

impl ChangeGainSTMSegment {
    pub const fn new(segment: Segment) -> Self {
        Self { segment }
    }

    pub const fn segment(&self) -> Segment {
        self.segment
    }
}

impl crate::datagram::Datagram for ChangeGainSTMSegment {
    type O1 = crate::operation::GainSTMChangeSegmentOp;
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
    use std::collections::HashMap;

    use autd3_derive::Gain;

    use super::*;

    use crate::{
        common::Drive,
        datagram::Gain,
        derive::*,
        geometry::Geometry,
        operation::{tests::NullGain, GainSTMOp, NullOp},
    };

    #[test]
    fn new() {
        let stm = GainSTM::<NullGain>::from_freq(1.)
            .add_gains_from_iter((0..10).map(|_| NullGain {}))
            .unwrap();

        assert_eq!(stm.frequency(), 1.);
        assert_eq!(stm.sampling_config().unwrap().frequency(), 1. * 10.);
    }

    #[test]
    fn from_period() {
        let stm = GainSTM::<NullGain>::from_period(std::time::Duration::from_micros(250))
            .add_gains_from_iter((0..10).map(|_| NullGain {}))
            .unwrap();

        assert_eq!(stm.period(), std::time::Duration::from_micros(250));
        assert_eq!(
            stm.sampling_config().unwrap().period(),
            std::time::Duration::from_micros(25)
        );
    }

    #[test]
    fn from_sampling_config() {
        let stm = GainSTM::<NullGain>::from_sampling_config(
            SamplingConfiguration::from_period(std::time::Duration::from_micros(25)).unwrap(),
        )
        .add_gains_from_iter((0..10).map(|_| NullGain {}))
        .unwrap();

        assert_eq!(stm.period(), std::time::Duration::from_micros(250));
        assert_eq!(
            stm.sampling_config().unwrap().period(),
            std::time::Duration::from_micros(25)
        );
    }

    #[test]
    fn with_mode() {
        let stm = GainSTM::<NullGain>::from_freq(1.0);
        assert_eq!(stm.mode(), GainSTMMode::PhaseIntensityFull);

        let stm = stm.with_mode(GainSTMMode::PhaseFull);
        assert_eq!(stm.mode(), GainSTMMode::PhaseFull);

        let stm = stm.with_mode(GainSTMMode::PhaseHalf);
        assert_eq!(stm.mode(), GainSTMMode::PhaseHalf);

        let stm = stm.with_mode(GainSTMMode::PhaseIntensityFull);
        assert_eq!(stm.mode(), GainSTMMode::PhaseIntensityFull);
    }

    #[derive(Gain)]
    struct NullGain2 {}

    impl Gain for NullGain2 {
        #[cfg_attr(coverage_nightly, coverage(off))]
        fn calc(
            &self,
            _: &Geometry,
            _: GainFilter,
        ) -> Result<HashMap<usize, Vec<Drive>>, AUTDInternalError> {
            unimplemented!()
        }
    }

    #[test]
    fn test_clear() {
        let mut stm = GainSTM::<Box<dyn Gain>>::from_freq(1.0)
            .add_gain(Box::new(NullGain {}))
            .unwrap()
            .add_gain(Box::new(NullGain2 {}))
            .unwrap();

        let gains = stm.clear();

        assert_eq!(stm.gains().len(), 0);
        assert_eq!(gains.len(), 2);
    }

    #[test]
    fn gain_stm_indexer() {
        let stm = GainSTM::from_freq(1.).add_gain(NullGain {}).unwrap();
        let _: &NullGain = &stm[0];
    }

    #[test]
    fn gain_stm_operation() {
        let stm = GainSTM::<Box<dyn Gain>>::from_freq(1.)
            .add_gain(Box::new(NullGain {}))
            .unwrap()
            .add_gain(Box::new(NullGain2 {}))
            .unwrap();

        assert_eq!(stm.timeout(), Some(std::time::Duration::from_millis(200)));

        let r = stm.operation_with_segment(Segment::S0, true);
        assert!(r.is_ok());
        let _: (GainSTMOp<Box<dyn Gain>>, NullOp) = r.unwrap();
    }
}
