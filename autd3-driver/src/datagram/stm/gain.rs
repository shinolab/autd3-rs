use crate::datagram::*;
use crate::{
    defined::Freq,
    derive::*,
    firmware::{cpu::GainSTMMode, fpga::STMSamplingConfig, operation::GainSTMOp},
};

#[derive(Builder)]
#[no_const]
pub struct GainSTM<G: Gain> {
    gains: Vec<G>,
    #[getset]
    loop_behavior: LoopBehavior,
    #[get]
    sampling_config: SamplingConfig,
    #[getset]
    mode: GainSTMMode,
}

impl<G: Gain> GainSTM<G> {
    pub fn from_freq<F: IntoIterator<Item = G> + Send + Sync + Clone>(
        freq: Freq<f64>,
        gains: F,
    ) -> Result<Self, AUTDInternalError> {
        let gains = gains.into_iter().collect::<Vec<_>>();
        Ok(Self {
            loop_behavior: LoopBehavior::infinite(),
            sampling_config: STMSamplingConfig::Freq(freq).sampling(gains.len())?,
            mode: GainSTMMode::PhaseIntensityFull,
            gains,
        })
    }

    pub fn from_freq_nearest<F: IntoIterator<Item = G> + Send + Sync + Clone>(
        freq: Freq<f64>,
        gains: F,
    ) -> Result<Self, AUTDInternalError> {
        let gains = gains.into_iter().collect::<Vec<_>>();
        Ok(Self {
            loop_behavior: LoopBehavior::infinite(),
            sampling_config: STMSamplingConfig::FreqNearest(freq).sampling(gains.len())?,
            mode: GainSTMMode::PhaseIntensityFull,
            gains,
        })
    }

    pub fn from_sampling_config<F: IntoIterator<Item = G> + Send + Sync + Clone>(
        config: SamplingConfig,
        gains: F,
    ) -> Self {
        Self {
            gains: gains.into_iter().collect::<Vec<_>>(),
            loop_behavior: LoopBehavior::infinite(),
            sampling_config: config,
            mode: GainSTMMode::PhaseIntensityFull,
        }
    }
}

pub struct GainSTMOperationGenerator {
    #[allow(clippy::type_complexity)]
    g: Vec<Box<dyn Fn(&Device) -> Box<dyn Fn(&Transducer) -> Drive + Sync + Send> + Send + Sync>>,
    mode: GainSTMMode,
    config: SamplingConfig,
    rep: u32,
    segment: Segment,
    transition_mode: Option<TransitionMode>,
}

impl OperationGenerator for GainSTMOperationGenerator {
    type O1 = GainSTMOp;
    type O2 = NullOp;

    fn generate(&self, device: &Device) -> (Self::O1, Self::O2) {
        let d = self.g.iter().map(|g| g(device)).collect::<Vec<_>>();
        (
            Self::O1::new(
                d,
                self.mode,
                self.config,
                self.rep,
                self.segment,
                self.transition_mode,
            ),
            Self::O2::default(),
        )
    }
}

impl<'a, G: Gain + 'a + 'a> DatagramST<'a> for GainSTM<G> {
    type O1 = GainSTMOp;
    type O2 = NullOp;
    type G = GainSTMOperationGenerator;

    fn operation_generator_with_segment(
        self,
        geometry: &Geometry,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> Result<Self::G, AUTDInternalError> {
        let sampling_config = self.sampling_config;
        let rep = self.loop_behavior.rep();
        let mode = self.mode;
        let gains = self
            .gains
            .into_iter()
            .map(|g| Ok(Box::new(g.calc(geometry)?) as Box<_>))
            .collect::<Result<Vec<_>, AUTDInternalError>>()?;
        Ok(GainSTMOperationGenerator {
            g: gains,
            mode,
            config: sampling_config,
            rep,
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

    use crate::defined::{kHz, Hz};

    #[derive(Gain)]
    struct Null {}

    impl Gain for Null {
        fn calc(&self, _geometry: &Geometry) -> GainCalcResult {
            Ok(Self::transform(|_| |_| Drive::null()))
        }
    }

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
            GainSTM::from_freq(freq, (0..n).map(|_| Null {})).map(|g| g.sampling_config())
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
            GainSTM::from_freq_nearest(freq, (0..n).map(|_| Null {})).map(|g| g.sampling_config())
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
            GainSTM::from_sampling_config(config, (0..n).map(|_| Null {})).sampling_config()
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
            GainSTM::from_sampling_config(SamplingConfig::Division(512), (0..2).map(|_| Null {}))
                .with_mode(mode)
                .mode()
        );
    }

    #[rstest::rstest]
    #[test]
    fn with_mode_default() {
        assert_eq!(
            GainSTMMode::PhaseIntensityFull,
            GainSTM::from_sampling_config(SamplingConfig::Division(512), (0..2).map(|_| Null {}))
                .mode()
        );
    }

    #[rstest::rstest]
    #[test]
    #[case::infinite(LoopBehavior::infinite())]
    #[case::finite(LoopBehavior::once())]
    fn with_loop_behavior(#[case] loop_behavior: LoopBehavior) {
        assert_eq!(
            loop_behavior,
            GainSTM::from_sampling_config(SamplingConfig::Division(512), (0..2).map(|_| Null {}))
                .with_loop_behavior(loop_behavior)
                .loop_behavior()
        );
    }
}
