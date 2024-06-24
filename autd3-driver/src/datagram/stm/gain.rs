use crate::datagram::*;
use crate::{
    defined::Freq,
    derive::*,
    firmware::{cpu::GainSTMMode, fpga::STMSamplingConfig, operation::GainSTMOp},
};

use derive_more::{Deref, DerefMut};

#[derive(Builder, Clone, Deref, DerefMut)]
pub struct GainSTM<G: Gain> {
    #[deref]
    #[deref_mut]
    gains: Vec<G>,
    #[getset]
    loop_behavior: LoopBehavior,
    #[get]
    sampling_config: SamplingConfig,
    #[getset]
    mode: GainSTMMode,
}

impl<G: Gain> GainSTM<G> {
    pub fn from_freq<F: IntoIterator<Item = G>>(
        freq: Freq<f32>,
        gains: F,
    ) -> Result<Self, AUTDInternalError> {
        let gains = gains.into_iter().collect::<Vec<_>>();
        Ok(Self::new(
            STMSamplingConfig::Freq(freq).sampling(gains.len())?,
            gains,
        ))
    }

    pub fn from_freq_nearest<F: IntoIterator<Item = G>>(
        freq: Freq<f32>,
        gains: F,
    ) -> Result<Self, AUTDInternalError> {
        let gains = gains.into_iter().collect::<Vec<_>>();
        Ok(Self::new(
            STMSamplingConfig::FreqNearest(freq).sampling(gains.len())?,
            gains,
        ))
    }

    pub fn from_period<F: IntoIterator<Item = G>>(
        period: Duration,
        gains: F,
    ) -> Result<Self, AUTDInternalError> {
        let gains = gains.into_iter().collect::<Vec<_>>();
        Ok(Self::new(
            STMSamplingConfig::Period(period).sampling(gains.len())?,
            gains,
        ))
    }

    pub fn from_period_nearest<F: IntoIterator<Item = G>>(
        period: Duration,
        gains: F,
    ) -> Result<Self, AUTDInternalError> {
        let gains = gains.into_iter().collect::<Vec<_>>();
        Ok(Self::new(
            STMSamplingConfig::PeriodNearest(period).sampling(gains.len())?,
            gains,
        ))
    }

    pub fn from_sampling_config<F: IntoIterator<Item = G>>(
        config: impl Into<SamplingConfig>,
        gains: F,
    ) -> Self {
        Self::new(config.into(), gains.into_iter().collect::<Vec<_>>())
    }

    #[cfg(feature = "capi")]
    pub fn from_stm_sampling_config<F: IntoIterator<Item = G>>(
        config: STMSamplingConfig,
        gains: F,
    ) -> Result<Self, AUTDInternalError> {
        let gains = gains.into_iter().collect::<Vec<_>>();
        Ok(Self::new(config.sampling(gains.len())?, gains))
    }

    fn new(sampling_config: SamplingConfig, gains: Vec<G>) -> Self {
        Self {
            gains,
            loop_behavior: LoopBehavior::infinite(),
            sampling_config,
            mode: GainSTMMode::PhaseIntensityFull,
        }
    }

    pub fn freq(&self) -> Result<Freq<f32>, AUTDInternalError> {
        self.sampling_config()
            .freq()
            .map(|f| f / self.gains.len() as f32)
    }

    pub fn period(&self) -> Result<Duration, AUTDInternalError> {
        self.sampling_config()
            .period()
            .map(|p| p * self.gains.len() as u32)
    }
}

pub struct GainSTMOperationGenerator<G: Gain> {
    pub gain: Vec<G>,
    #[allow(clippy::type_complexity)]
    g: *const Vec<Box<dyn Fn(&Device) -> Box<dyn Fn(&Transducer) -> Drive + Sync + Send>>>,
    mode: GainSTMMode,
    config: SamplingConfig,
    rep: u32,
    segment: Segment,
    transition_mode: Option<TransitionMode>,
}

impl<G: Gain> GainSTMOperationGenerator<G> {
    fn new(
        g: Vec<G>,
        geometry: &Geometry,
        mode: GainSTMMode,
        config: SamplingConfig,
        rep: u32,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> Result<Self, AUTDInternalError> {
        let mut r = Self {
            gain: g,
            g: std::ptr::null(),
            mode,
            config,
            rep,
            segment,
            transition_mode,
        };
        r.g = Box::into_raw(Box::new(
            r.gain
                .iter()
                .map(|g| {
                    Ok(Box::new(g.calc(geometry)?)
                        as Box<
                            dyn Fn(&Device) -> Box<dyn Fn(&Transducer) -> Drive + Sync + Send>,
                        >)
                })
                .collect::<Result<Vec<_>, AUTDInternalError>>()?,
        )) as *const _;
        Ok(r)
    }
}

impl<G: Gain> Drop for GainSTMOperationGenerator<G> {
    fn drop(&mut self) {
        unsafe {
            let _ = Box::from_raw(
                self.g
                    as *mut Vec<
                        Box<dyn Fn(&Device) -> Box<dyn Fn(&Transducer) -> Drive + Sync + Send>>,
                    >,
            );
        }
    }
}

impl<G: Gain> OperationGenerator for GainSTMOperationGenerator<G> {
    type O1 = GainSTMOp;
    type O2 = NullOp;

    fn generate(&self, device: &Device) -> (Self::O1, Self::O2) {
        let d = unsafe { (*self.g).iter().map(|g| g(device)).collect::<Vec<_>>() };
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

impl<G: Gain> DatagramST for GainSTM<G> {
    type G = GainSTMOperationGenerator<G>;

    fn operation_generator_with_segment(
        self,
        geometry: &Geometry,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> Result<Self::G, AUTDInternalError> {
        let sampling_config = self.sampling_config;
        let rep = self.loop_behavior.rep();
        let mode = self.mode;
        let gains = self.gains;
        GainSTMOperationGenerator::new(
            gains,
            geometry,
            mode,
            sampling_config,
            rep,
            segment,
            transition_mode,
        )
    }

    fn timeout(&self) -> Option<std::time::Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    #[tracing::instrument(level = "debug", skip(self, geometry), fields(%self.loop_behavior, %self.sampling_config, ?self.mode))]
    // GRCOV_EXCL_START
    fn trace(&self, geometry: &Geometry) {
        tracing::debug!("{}", tynm::type_name::<Self>());
        if tracing::enabled!(tracing::Level::DEBUG) {
            if tracing::enabled!(tracing::Level::TRACE) {
                self.gains.iter().enumerate().for_each(|(i, g)| {
                    tracing::debug!("Gain[{}]", i);
                    g.trace(geometry);
                });
            } else {
                let len = self.gains.len();
                tracing::debug!("Gain[{}]", 0);
                self.gains[0].trace(geometry);
                if len > 2 {
                    tracing::debug!("︙");
                }
                if len > 1 {
                    tracing::debug!("Gain[{}]", len - 1);
                    self.gains[len - 1].trace(geometry);
                }
            }
        }
    }
    // GRCOV_EXCL_STOP
}

#[cfg(feature = "capi")]
impl Default for GainSTM<Box<dyn Gain + Send + Sync>> {
    fn default() -> Self {
        Self {
            gains: vec![],
            loop_behavior: LoopBehavior::infinite(),
            sampling_config: SamplingConfig::DISABLE,
            mode: GainSTMMode::PhaseIntensityFull,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::defined::{kHz, Hz};

    #[derive(Gain, Default, Debug, PartialEq)]
    struct Null {
        i: i32,
    }

    impl Gain for Null {
        // GRCOV_EXCL_START
        fn calc(&self, _geometry: &Geometry) -> GainCalcResult {
            Ok(Self::transform(|_| |_| Drive::null()))
        }
        // GRCOV_EXCL_STOP
    }

    #[rstest::rstest]
    #[test]
    #[case(Ok(SamplingConfig::Freq(1*Hz)), 0.5*Hz, 2)]
    #[case(Ok(SamplingConfig::Freq(10*Hz)), 1.*Hz, 10)]
    #[case(Ok(SamplingConfig::Freq(20*Hz)), 2.*Hz, 10)]
    #[case(Err(AUTDInternalError::STMFreqInvalid(2, 0.49*Hz)), 0.49*Hz, 2)]
    fn from_freq(
        #[case] expect: Result<SamplingConfig, AUTDInternalError>,
        #[case] freq: Freq<f32>,
        #[case] n: usize,
    ) {
        assert_eq!(
            expect,
            GainSTM::from_freq(freq, (0..n).map(|_| Null::default())).map(|g| g.sampling_config())
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
        #[case] freq: Freq<f32>,
        #[case] n: usize,
    ) {
        assert_eq!(
            expect,
            GainSTM::from_freq_nearest(freq, (0..n).map(|_| Null::default()))
                .map(|g| g.sampling_config())
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(
        Ok(SamplingConfig::Period(Duration::from_millis(1000))),
        Duration::from_millis(2000),
        2
    )]
    #[case(
        Ok(SamplingConfig::Period(Duration::from_millis(100))),
        Duration::from_millis(1000),
        10
    )]
    #[case(
        Ok(SamplingConfig::Period(Duration::from_millis(50))),
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
            GainSTM::from_period(p, (0..n).map(|_| Null::default())).map(|f| f.sampling_config())
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(
        Ok(SamplingConfig::PeriodNearest(Duration::from_millis(1000))),
        Duration::from_millis(2000),
        2
    )]
    #[case(
        Ok(SamplingConfig::PeriodNearest(Duration::from_millis(100))),
        Duration::from_millis(1000),
        10
    )]
    #[case(
        Ok(SamplingConfig::PeriodNearest(Duration::from_millis(50))),
        Duration::from_millis(500),
        10
    )]
    #[case(Ok(SamplingConfig::PeriodNearest(Duration::from_millis(1000))), Duration::from_millis(2000) + Duration::from_nanos(1), 2)]
    fn from_period_nearest(
        #[case] expect: Result<SamplingConfig, AUTDInternalError>,
        #[case] p: Duration,
        #[case] n: usize,
    ) {
        assert_eq!(
            expect,
            GainSTM::from_period_nearest(p, (0..n).map(|_| Null::default()))
                .map(|f| f.sampling_config())
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
            GainSTM::from_sampling_config(config, (0..n).map(|_| Null::default()))
                .sampling_config()
        );
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    #[case(Ok(0.5*Hz), 0.5*Hz, 2)]
    #[case(Ok(1.0*Hz), 1.*Hz, 10)]
    #[case(Ok(2.0*Hz), 2.*Hz, 10)]
    #[case(Err(AUTDInternalError::STMFreqInvalid(2, 0.49*Hz)), 0.49*Hz, 2)]
    fn freq(
        #[case] expect: Result<Freq<f32>, AUTDInternalError>,
        #[case] f: Freq<f32>,
        #[case] n: usize,
    ) {
        assert_eq!(
            expect,
            GainSTM::from_freq(f, (0..n).map(|_| Null::default())).and_then(|f| f.freq())
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(Ok(Duration::from_millis(2000)), 0.5*Hz, 2)]
    #[case(Ok(Duration::from_millis(1000)), 1.*Hz, 10)]
    #[case(Ok(Duration::from_millis(500)), 2.*Hz, 10)]
    #[case(Err(AUTDInternalError::STMFreqInvalid(2, 0.49*Hz)), 0.49*Hz, 2)]
    fn period(
        #[case] expect: Result<Duration, AUTDInternalError>,
        #[case] f: Freq<f32>,
        #[case] n: usize,
    ) {
        assert_eq!(
            expect,
            GainSTM::from_freq(f, (0..n).map(|_| Null::default())).and_then(|f| f.period())
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
            GainSTM::from_sampling_config(
                SamplingConfig::Division(512),
                (0..2).map(|_| Null::default())
            )
            .with_mode(mode)
            .mode()
        );
    }

    #[rstest::rstest]
    #[test]
    fn with_mode_default() {
        assert_eq!(
            GainSTMMode::PhaseIntensityFull,
            GainSTM::from_sampling_config(
                SamplingConfig::Division(512),
                (0..2).map(|_| Null::default())
            )
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
            GainSTM::from_sampling_config(
                SamplingConfig::Division(512),
                (0..2).map(|_| Null::default())
            )
            .with_loop_behavior(loop_behavior)
            .loop_behavior()
        );
    }
}
