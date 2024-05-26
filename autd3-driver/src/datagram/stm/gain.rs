use crate::{
    defined::{Freq, DEFAULT_TIMEOUT},
    derive::*,
    firmware::{
        cpu::GainSTMMode,
        fpga::{STMSamplingConfig, Segment, TransitionMode},
    },
};

#[derive(Builder)]
#[no_const]
pub struct GainSTM<G: Gain> {
    gains: Vec<G>,
    #[getset]
    loop_behavior: LoopBehavior,
    #[get]
    stm_sampling_config: STMSamplingConfig,
    #[getset]
    mode: GainSTMMode,
}

impl<G: Gain> GainSTM<G> {
    pub const fn from_freq(freq: Freq<f64>) -> Self {
        Self {
            gains: Vec::new(),
            loop_behavior: LoopBehavior::infinite(),
            stm_sampling_config: STMSamplingConfig::Freq(freq),
            mode: GainSTMMode::PhaseIntensityFull,
        }
    }

    pub const fn from_freq_nearest(freq: Freq<f64>) -> Self {
        Self {
            gains: Vec::new(),
            loop_behavior: LoopBehavior::infinite(),
            stm_sampling_config: STMSamplingConfig::FreqNearest(freq),
            mode: GainSTMMode::PhaseIntensityFull,
        }
    }

    pub const fn from_sampling_config(config: SamplingConfig) -> Self {
        Self {
            gains: Vec::new(),
            loop_behavior: LoopBehavior::infinite(),
            stm_sampling_config: STMSamplingConfig::SamplingConfig(config),
            mode: GainSTMMode::PhaseIntensityFull,
        }
    }

    pub fn add_gain(mut self, gain: G) -> Self {
        self.gains.push(gain);
        self
    }

    pub fn add_gains_from_iter(mut self, iter: impl IntoIterator<Item = G>) -> Self {
        self.gains.extend(iter);
        self
    }

    pub fn clear(&mut self) -> Vec<G> {
        std::mem::take(&mut self.gains)
    }

    pub fn sampling_config(&self) -> Result<SamplingConfig, AUTDInternalError> {
        self.stm_sampling_config.sampling(self.gains.len())
    }
}

impl<G: Gain> std::ops::Deref for GainSTM<G> {
    type Target = [G];

    fn deref(&self) -> &Self::Target {
        &self.gains
    }
}

impl<G: Gain> std::ops::DerefMut for GainSTM<G> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.gains
    }
}

impl<G: Gain> DatagramST for GainSTM<G> {
    type O1 = crate::firmware::operation::GainSTMOp<G>;
    type O2 = crate::firmware::operation::NullOp;

    fn operation_with_segment(
        self,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> (Self::O1, Self::O2) {
        let Self {
            gains,
            mode,
            loop_behavior,
            stm_sampling_config,
        } = self;
        (
            Self::O1::new(
                gains,
                mode,
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

// GRCOV_EXCL_START
impl<G: Gain + Clone> Clone for GainSTM<G> {
    fn clone(&self) -> Self {
        Self {
            gains: self.gains.clone(),
            mode: self.mode,
            loop_behavior: self.loop_behavior,
            stm_sampling_config: self.stm_sampling_config,
        }
    }
}
// GRCOV_EXCL_STOP

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use autd3_derive::Gain;

    use super::*;

    use crate::{
        defined::{kHz, Hz},
        firmware::operation::{tests::NullGain, GainSTMOp},
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
            GainSTM::from_freq(freq)
                .add_gains_from_iter((0..n).map(|_| NullGain {}))
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
            GainSTM::from_freq_nearest(freq)
                .add_gains_from_iter((0..n).map(|_| NullGain {}))
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
            GainSTM::from_sampling_config(config)
                .add_gains_from_iter((0..n).map(|_| NullGain {}))
                .sampling_config()?
        );
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    #[case::phase_intensity_full(GainSTMMode::PhaseIntensityFull)]
    #[case::phase_full(GainSTMMode::PhaseFull)]
    #[case::phase_half(GainSTMMode::PhaseHalf)]
    fn with_mode(#[case] mode: GainSTMMode) {
        assert_eq!(
            mode,
            GainSTM::<NullGain>::from_freq(1. * Hz)
                .with_mode(mode)
                .mode()
        );
    }

    #[test]
    fn with_mode_default() {
        assert_eq!(
            GainSTMMode::PhaseIntensityFull,
            GainSTM::<NullGain>::from_freq(1. * Hz).mode()
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
    fn clear() {
        let mut stm = GainSTM::<Box<dyn Gain>>::from_freq(1. * Hz)
            .add_gain(Box::new(NullGain {}))
            .add_gain(Box::new(NullGain2 {}));
        assert_eq!(stm.len(), 2);
        let gains = stm.clear();
        assert_eq!(stm.len(), 0);
        assert_eq!(gains.len(), 2);
    }

    #[test]
    fn deref_mut() {
        let mut stm = GainSTM::<Box<dyn Gain>>::from_freq(1. * Hz).add_gain(Box::new(NullGain {}));
        stm[0] = Box::new(NullGain2 {});
    }

    #[rstest::rstest]
    #[test]
    #[case::infinite(LoopBehavior::infinite())]
    #[case::finite(LoopBehavior::once())]
    fn with_loop_behavior(#[case] loop_behavior: LoopBehavior) {
        assert_eq!(
            loop_behavior,
            GainSTM::<Box<dyn Gain>>::from_freq(1. * Hz)
                .with_loop_behavior(loop_behavior)
                .loop_behavior()
        );
    }

    #[test]
    fn operation() {
        let stm = GainSTM::<Box<dyn Gain>>::from_freq(1. * Hz)
            .add_gain(Box::new(NullGain {}))
            .add_gain(Box::new(NullGain2 {}));

        assert_eq!(Datagram::timeout(&stm), Some(DEFAULT_TIMEOUT));

        let _: (GainSTMOp<Box<dyn Gain>>, NullOp) =
            stm.operation_with_segment(Segment::S0, Some(TransitionMode::SyncIdx));
    }
}
