use crate::{
    derive::*,
    fpga::{Segment, TransitionMode},
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
#[derive(Builder)]
#[no_const]
pub struct GainSTM<G: Gain> {
    gains: Vec<G>,
    #[getset(loop_behavior: LoopBehavior)]
    props: STMProps,
    #[getset]
    mode: GainSTMMode,
}

impl<G: Gain> GainSTM<G> {
    /// constructor
    ///
    /// # Arguments
    ///
    /// * `freq` - Frequency of STM. The frequency closest to `freq` from the possible frequencies is set.
    ///
    pub const fn from_freq(freq: f64) -> Self {
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

    pub fn frequency(&self) -> f64 {
        self.props.freq(self.gains.len())
    }

    pub fn period(&self) -> std::time::Duration {
        self.props.period(self.gains.len())
    }

    pub fn sampling_config(&self) -> Result<SamplingConfiguration, AUTDInternalError> {
        self.props.sampling_config(self.gains.len())
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
        segment: Segment,
        transition_mode: TransitionMode,
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
                transition_mode,
                update_segment,
            ),
            Self::O2::default(),
        ))
    }

    fn timeout(&self) -> Option<std::time::Duration> {
        Some(std::time::Duration::from_millis(200))
    }
}

impl<G: Gain> DatagramT for GainSTM<G> {}

impl<G: Gain + Clone> Clone for GainSTM<G> {
    fn clone(&self) -> Self {
        Self {
            gains: self.gains.clone(),
            mode: self.mode,
            props: self.props,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, time::Duration};

    use autd3_derive::Gain;

    use super::*;

    use crate::operation::{tests::NullGain, GainSTMOp};

    #[rstest::rstest]
    #[test]
    #[case(0.5, 2)]
    #[case(1.0, 10)]
    #[case(2.0, 10)]
    fn test_from_requency(#[case] freq: f64, #[case] n: usize) -> anyhow::Result<()> {
        let stm = GainSTM::from_freq(freq).add_gains_from_iter((0..n).map(|_| NullGain {}))?;
        assert_eq!(freq, stm.frequency());
        assert_eq!(freq * n as f64, stm.sampling_config()?.frequency());
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    #[case(Duration::from_micros(125), 2)]
    #[case(Duration::from_micros(250), 10)]
    #[case(Duration::from_micros(500), 10)]
    fn test_from_period(#[case] period: Duration, #[case] n: usize) -> anyhow::Result<()> {
        let stm = GainSTM::from_period(period).add_gains_from_iter((0..n).map(|_| NullGain {}))?;
        assert_eq!(period, stm.period());
        assert_eq!(period / n as u32, stm.sampling_config()?.period());
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    #[case(SamplingConfiguration::DISABLE, 2)]
    #[case(SamplingConfiguration::FREQ_4K_HZ, 10)]
    fn test_from_sampling_config(
        #[case] config: SamplingConfiguration,
        #[case] n: usize,
    ) -> anyhow::Result<()> {
        assert_eq!(
            config,
            GainSTM::from_sampling_config(config)
                .add_gains_from_iter((0..n).map(|_| NullGain {}))?
                .sampling_config()?
        );
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    #[case::phase_intensity_full(GainSTMMode::PhaseIntensityFull)]
    #[case::phase_full(GainSTMMode::PhaseFull)]
    #[case::phase_half(GainSTMMode::PhaseHalf)]
    fn test_with_mode(#[case] mode: GainSTMMode) {
        assert_eq!(
            mode,
            GainSTM::<NullGain>::from_freq(1.0).with_mode(mode).mode()
        );
    }

    #[test]
    fn test_with_mode_default() {
        assert_eq!(
            GainSTMMode::PhaseIntensityFull,
            GainSTM::<NullGain>::from_freq(1.0).mode()
        );
    }

    #[derive(Gain)]
    struct NullGain2 {}

    impl Gain for NullGain2 {
        // GRCOV_EXCL_START
        fn calc(
            &self,
            _: &Geometry,
            _: GainFilter,
        ) -> Result<HashMap<usize, Vec<Drive>>, AUTDInternalError> {
            unimplemented!()
        }
        // GRCOV_EXCL_STOP
    }

    #[test]
    fn test_clear() -> anyhow::Result<()> {
        let mut stm = GainSTM::<Box<dyn Gain>>::from_freq(1.0)
            .add_gain(Box::new(NullGain {}))?
            .add_gain(Box::new(NullGain2 {}))?;
        assert_eq!(stm.gains().len(), 2);
        let gains = stm.clear();
        assert_eq!(stm.gains().len(), 0);
        assert_eq!(gains.len(), 2);
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    #[case::infinite(LoopBehavior::Infinite)]
    #[case::finite(LoopBehavior::once())]
    fn test_with_loop_behavior(#[case] loop_behavior: LoopBehavior) {
        assert_eq!(
            loop_behavior,
            GainSTM::<Box<dyn Gain>>::from_freq(1.0)
                .with_loop_behavior(loop_behavior)
                .loop_behavior()
        );
    }

    #[test]
    fn test_indexer() {
        let stm = GainSTM::from_freq(1.).add_gain(NullGain {}).unwrap();
        let _: &NullGain = &stm[0];
    }

    #[test]
    fn test_operation() {
        let stm = GainSTM::<Box<dyn Gain>>::from_freq(1.)
            .add_gain(Box::new(NullGain {}))
            .unwrap()
            .add_gain(Box::new(NullGain2 {}))
            .unwrap();

        assert_eq!(stm.timeout(), Some(std::time::Duration::from_millis(200)));

        let r = stm.operation_with_segment(Segment::S0, TransitionMode::SyncIdx, true);
        assert!(r.is_ok());
        let _: (GainSTMOp<Box<dyn Gain>>, NullOp) = r.unwrap();
    }
}
