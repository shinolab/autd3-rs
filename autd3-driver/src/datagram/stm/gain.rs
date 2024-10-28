use super::sampling_config::*;
use crate::{
    datagram::*,
    defined::Freq,
    derive::*,
    firmware::{cpu::GainSTMMode, operation::GainSTMOp},
};

use derive_more::{Deref, DerefMut};
use silencer::WithSampling;

#[derive(Builder, Clone, Deref, DerefMut, Debug)]
pub struct GainSTM<G: Gain> {
    #[deref]
    #[deref_mut]
    gains: Vec<G>,
    #[get]
    #[set]
    loop_behavior: LoopBehavior,
    #[get]
    sampling_config: SamplingConfig,
    #[get]
    #[set]
    mode: GainSTMMode,
}

impl<G: Gain> WithSampling for GainSTM<G> {
    fn sampling_config_intensity(&self) -> Option<SamplingConfig> {
        Some(self.sampling_config)
    }

    fn sampling_config_phase(&self) -> Option<SamplingConfig> {
        Some(self.sampling_config)
    }
}

impl<G: Gain> GainSTM<G> {
    pub fn new<F: IntoIterator<Item = G>>(
        config: impl Into<STMConfig>,
        gains: F,
    ) -> Result<Self, AUTDInternalError> {
        Self::new_from_sampling_config(config.into(), gains)
    }

    pub fn new_nearest<F: IntoIterator<Item = G>>(
        config: impl Into<STMConfigNearest>,
        gains: F,
    ) -> Result<Self, AUTDInternalError> {
        Self::new_from_sampling_config(config.into(), gains)
    }

    fn new_from_sampling_config<T, F: IntoIterator<Item = G>>(
        config: T,
        gains: F,
    ) -> Result<Self, AUTDInternalError>
    where
        SamplingConfig: TryFrom<(T, usize), Error = AUTDInternalError>,
    {
        let gains = gains.into_iter().collect::<Vec<_>>();
        if gains.is_empty() {
            return Err(AUTDInternalError::GainSTMSizeOutOfRange(gains.len()));
        }
        Ok(Self {
            sampling_config: (config, gains.len()).try_into()?,
            gains,
            loop_behavior: LoopBehavior::infinite(),
            mode: GainSTMMode::PhaseIntensityFull,
        })
    }

    pub fn freq(&self) -> Freq<f32> {
        self.sampling_config().freq() / self.gains.len() as f32
    }

    pub fn period(&self) -> Duration {
        self.sampling_config().period() * self.gains.len() as u32
    }
}

pub struct GainSTMOperationGenerator<G: GainContextGenerator> {
    g: Vec<G>,
    mode: GainSTMMode,
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
    segment: Segment,
    transition_mode: Option<TransitionMode>,
}

impl<G: GainContextGenerator> GainSTMOperationGenerator<G> {
    fn new<T: Gain<G = G>>(
        gains: Vec<T>,
        geometry: &Geometry,
        mode: GainSTMMode,
        config: SamplingConfig,
        loop_behavior: LoopBehavior,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> Result<Self, AUTDInternalError> {
        Ok(Self {
            g: gains
                .into_iter()
                .map(|gain| gain.init(geometry))
                .collect::<Result<Vec<_>, AUTDInternalError>>()?,
            mode,
            config,
            loop_behavior,
            segment,
            transition_mode,
        })
    }
}

impl<G: GainContextGenerator> OperationGenerator for GainSTMOperationGenerator<G> {
    type O1 = GainSTMOp<G::Context>;
    type O2 = NullOp;

    fn generate(&mut self, device: &Device) -> (Self::O1, Self::O2) {
        let d = self
            .g
            .iter_mut()
            .map(|g| g.generate(device))
            .collect::<Vec<_>>();
        (
            Self::O1::new(
                d,
                self.mode,
                self.config,
                self.loop_behavior,
                self.segment,
                self.transition_mode,
            ),
            Self::O2::new(),
        )
    }
}

impl<G: Gain> DatagramS for GainSTM<G> {
    type G = GainSTMOperationGenerator<G::G>;

    fn operation_generator_with_segment(
        self,
        geometry: &Geometry,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> Result<Self::G, AUTDInternalError> {
        let sampling_config = self.sampling_config;
        let loop_behavior = self.loop_behavior;
        let mode = self.mode;
        let gains = self.gains;
        GainSTMOperationGenerator::new(
            gains,
            geometry,
            mode,
            sampling_config,
            loop_behavior,
            segment,
            transition_mode,
        )
    }

    fn parallel_threshold(&self) -> Option<usize> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::defined::{kHz, Hz};
    use gain::tests::TestGain;

    #[rstest::rstest]
    #[test]
    #[case((1. * Hz).try_into(), 0.5*Hz, 2)]
    #[case((10. * Hz).try_into(), 1.*Hz, 10)]
    #[case((20. * Hz).try_into(), 2.*Hz, 10)]
    #[case((2. * 0.49*Hz).try_into(), 0.49*Hz, 2)]
    #[case(Err(AUTDInternalError::GainSTMSizeOutOfRange(0)), 1.*Hz, 0)]
    fn from_freq(
        #[case] expect: Result<SamplingConfig, AUTDInternalError>,
        #[case] freq: Freq<f32>,
        #[case] n: usize,
    ) {
        assert_eq!(
            expect,
            GainSTM::new(freq, (0..n).map(|_| TestGain::null())).map(|g| g.sampling_config())
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(Ok(SamplingConfig::new_nearest(1. * Hz)), 0.5*Hz, 2)]
    #[case(Ok(SamplingConfig::new_nearest(0.98 * Hz)), 0.49*Hz, 2)]
    #[case(Ok(SamplingConfig::new_nearest(10. * Hz)), 1.*Hz, 10)]
    #[case(Ok(SamplingConfig::new_nearest(20. * Hz)), 2.*Hz, 10)]
    fn from_freq_nearest(
        #[case] expect: Result<SamplingConfig, AUTDInternalError>,
        #[case] freq: Freq<f32>,
        #[case] n: usize,
    ) {
        assert_eq!(
            expect,
            GainSTM::new_nearest(freq, (0..n).map(|_| TestGain::null()))
                .map(|g| g.sampling_config())
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(
        Duration::from_millis(1000).try_into(),
        Duration::from_millis(2000),
        2
    )]
    #[case(
        Duration::from_millis(100).try_into(),
        Duration::from_millis(1000),
        10
    )]
    #[case(
        Duration::from_millis(50).try_into(),
        Duration::from_millis(500),
        10
    )]
    #[case(Err(AUTDInternalError::STMPeriodInvalid(2, Duration::from_millis(2000) + Duration::from_nanos(1))), Duration::from_millis(2000) + Duration::from_nanos(1), 2)]
    fn from_period(
        #[case] expect: Result<SamplingConfig, AUTDInternalError>,
        #[case] p: Duration,
        #[case] n: usize,
    ) {
        assert_eq!(
            expect,
            GainSTM::new(p, (0..n).map(|_| TestGain::null())).map(|f| f.sampling_config())
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(
        Duration::from_millis(1000).try_into(),
        Duration::from_millis(2000),
        2
    )]
    #[case(
        Duration::from_millis(100).try_into(),
        Duration::from_millis(1000),
        10
    )]
    #[case(
        Duration::from_millis(50).try_into(),
        Duration::from_millis(500),
        10
    )]
    #[case(Duration::from_millis(1000).try_into(), Duration::from_millis(2000) + Duration::from_nanos(1), 2)]
    fn from_period_nearest(
        #[case] expect: Result<SamplingConfig, AUTDInternalError>,
        #[case] p: Duration,
        #[case] n: usize,
    ) {
        assert_eq!(
            expect,
            GainSTM::new_nearest(p, (0..n).map(|_| TestGain::null())).map(|f| f.sampling_config())
        );
    }

    #[rstest::rstest]
    #[test]
    #[case((4. * kHz).try_into().unwrap(), 10)]
    #[case((8. * kHz).try_into().unwrap(), 10)]
    fn from_sampling_config(
        #[case] config: SamplingConfig,
        #[case] n: usize,
    ) -> anyhow::Result<()> {
        assert_eq!(
            Ok(config),
            GainSTM::new(config, (0..n).map(|_| TestGain::null())).map(|f| f.sampling_config())
        );
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    #[case(Ok(0.5*Hz), 0.5*Hz, 2)]
    #[case(Ok(1.0*Hz), 1.*Hz, 10)]
    #[case(Ok(2.0*Hz), 2.*Hz, 10)]
    fn freq(
        #[case] expect: Result<Freq<f32>, AUTDInternalError>,
        #[case] f: Freq<f32>,
        #[case] n: usize,
    ) {
        assert_eq!(
            expect,
            GainSTM::new(f, (0..n).map(|_| TestGain::null())).map(|f| f.freq())
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(Ok(Duration::from_millis(2000)), 0.5*Hz, 2)]
    #[case(Ok(Duration::from_millis(1000)), 1.*Hz, 10)]
    #[case(Ok(Duration::from_millis(500)), 2.*Hz, 10)]
    fn period(
        #[case] expect: Result<Duration, AUTDInternalError>,
        #[case] f: Freq<f32>,
        #[case] n: usize,
    ) {
        assert_eq!(
            expect,
            GainSTM::new(f, (0..n).map(|_| TestGain::null())).map(|f| f.period())
        );
    }

    #[rstest::rstest]
    #[test]
    #[case::phase_intensity_full(GainSTMMode::PhaseIntensityFull)]
    #[case::phase_full(GainSTMMode::PhaseFull)]
    #[case::phase_half(GainSTMMode::PhaseHalf)]
    fn with_mode(#[case] mode: GainSTMMode) {
        assert_eq!(
            mode,
            GainSTM::new(SamplingConfig::FREQ_40K, (0..2).map(|_| TestGain::null()))
                .unwrap()
                .with_mode(mode)
                .mode()
        );
    }

    #[rstest::rstest]
    #[test]
    fn with_mode_default() {
        assert_eq!(
            GainSTMMode::PhaseIntensityFull,
            GainSTM::new(SamplingConfig::FREQ_40K, (0..2).map(|_| TestGain::null()))
                .unwrap()
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
            GainSTM::new(SamplingConfig::FREQ_40K, (0..2).map(|_| TestGain::null()))
                .unwrap()
                .with_loop_behavior(loop_behavior)
                .loop_behavior()
        );
    }
}
