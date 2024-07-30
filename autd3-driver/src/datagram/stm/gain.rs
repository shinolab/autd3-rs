use super::sampling_config::*;
use crate::{
    datagram::*,
    defined::Freq,
    derive::*,
    firmware::{cpu::GainSTMMode, operation::GainSTMOp},
};

use derive_more::{Deref, DerefMut};

#[derive(Builder, Clone, Deref, DerefMut)]
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

pub struct GainSTMOperationGenerator<G: Gain> {
    pub gain: Vec<G>,
    #[allow(clippy::type_complexity)]
    g: *const Vec<Box<dyn Fn(&Device) -> Box<dyn Fn(&Transducer) -> Drive + Sync + Send>>>,
    mode: GainSTMMode,
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
    segment: Segment,
    transition_mode: Option<TransitionMode>,
}

impl<G: Gain> GainSTMOperationGenerator<G> {
    fn new(
        g: Vec<G>,
        geometry: &Geometry,
        mode: GainSTMMode,
        config: SamplingConfig,
        loop_behavior: LoopBehavior,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> Result<Self, AUTDInternalError> {
        let mut r = Self {
            gain: g,
            g: std::ptr::null(),
            mode,
            config,
            loop_behavior,
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
                self.loop_behavior,
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
            sampling_config: SamplingConfig::FREQ_40K,
            mode: GainSTMMode::PhaseIntensityFull,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        defined::{kHz, Hz},
        firmware::fpga::IntoSamplingConfigNearest,
    };

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
    #[case((1. * Hz).into_sampling_config(), 0.5*Hz, 2)]
    #[case((10. * Hz).into_sampling_config(), 1.*Hz, 10)]
    #[case((20. * Hz).into_sampling_config(), 2.*Hz, 10)]
    #[case((2. * 0.49*Hz).into_sampling_config(), 0.49*Hz, 2)]
    #[case(Err(AUTDInternalError::GainSTMSizeOutOfRange(0)), 1.*Hz, 0)]
    #[cfg_attr(miri, ignore)]
    fn from_freq(
        #[case] expect: Result<SamplingConfig, AUTDInternalError>,
        #[case] freq: Freq<f32>,
        #[case] n: usize,
    ) {
        assert_eq!(
            expect,
            GainSTM::new(freq, (0..n).map(|_| Null::default())).map(|g| g.sampling_config())
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(Ok((1. * Hz).into_sampling_config_nearest()), 0.5*Hz, 2)]
    #[case(Ok((0.98 * Hz).into_sampling_config_nearest()), 0.49*Hz, 2)]
    #[case(Ok((10. * Hz).into_sampling_config_nearest()), 1.*Hz, 10)]
    #[case(Ok((20. * Hz).into_sampling_config_nearest()), 2.*Hz, 10)]
    #[cfg_attr(miri, ignore)]
    fn from_freq_nearest(
        #[case] expect: Result<SamplingConfig, AUTDInternalError>,
        #[case] freq: Freq<f32>,
        #[case] n: usize,
    ) {
        assert_eq!(
            expect,
            GainSTM::new_nearest(freq, (0..n).map(|_| Null::default()))
                .map(|g| g.sampling_config())
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(
        Duration::from_millis(1000).into_sampling_config(),
        Duration::from_millis(2000),
        2
    )]
    #[case(
        Duration::from_millis(100).into_sampling_config(),
        Duration::from_millis(1000),
        10
    )]
    #[case(
        Duration::from_millis(50).into_sampling_config(),
        Duration::from_millis(500),
        10
    )]
    #[case(Err(AUTDInternalError::STMPeriodInvalid(2, Duration::from_millis(2000) + Duration::from_nanos(1))), Duration::from_millis(2000) + Duration::from_nanos(1), 2)]
    #[cfg_attr(miri, ignore)]
    fn from_period(
        #[case] expect: Result<SamplingConfig, AUTDInternalError>,
        #[case] p: Duration,
        #[case] n: usize,
    ) {
        assert_eq!(
            expect,
            GainSTM::new(p, (0..n).map(|_| Null::default())).map(|f| f.sampling_config())
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(
        Duration::from_millis(1000).into_sampling_config(),
        Duration::from_millis(2000),
        2
    )]
    #[case(
        Duration::from_millis(100).into_sampling_config(),
        Duration::from_millis(1000),
        10
    )]
    #[case(
        Duration::from_millis(50).into_sampling_config(),
        Duration::from_millis(500),
        10
    )]
    #[case(Duration::from_millis(1000).into_sampling_config(), Duration::from_millis(2000) + Duration::from_nanos(1), 2)]
    #[cfg_attr(miri, ignore)]
    fn from_period_nearest(
        #[case] expect: Result<SamplingConfig, AUTDInternalError>,
        #[case] p: Duration,
        #[case] n: usize,
    ) {
        assert_eq!(
            expect,
            GainSTM::new_nearest(p, (0..n).map(|_| Null::default())).map(|f| f.sampling_config())
        );
    }

    #[rstest::rstest]
    #[test]
    #[case((4. * kHz).into_sampling_config().unwrap(), 10)]
    #[case((8. * kHz).into_sampling_config().unwrap(), 10)]
    #[cfg_attr(miri, ignore)]
    fn from_sampling_config(
        #[case] config: SamplingConfig,
        #[case] n: usize,
    ) -> anyhow::Result<()> {
        assert_eq!(
            Ok(config),
            GainSTM::new(config, (0..n).map(|_| Null::default())).map(|f| f.sampling_config())
        );
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    #[case(Ok(0.5*Hz), 0.5*Hz, 2)]
    #[case(Ok(1.0*Hz), 1.*Hz, 10)]
    #[case(Ok(2.0*Hz), 2.*Hz, 10)]
    #[cfg_attr(miri, ignore)]
    fn freq(
        #[case] expect: Result<Freq<f32>, AUTDInternalError>,
        #[case] f: Freq<f32>,
        #[case] n: usize,
    ) {
        assert_eq!(
            expect,
            GainSTM::new(f, (0..n).map(|_| Null::default())).map(|f| f.freq())
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(Ok(Duration::from_millis(2000)), 0.5*Hz, 2)]
    #[case(Ok(Duration::from_millis(1000)), 1.*Hz, 10)]
    #[case(Ok(Duration::from_millis(500)), 2.*Hz, 10)]
    #[cfg_attr(miri, ignore)]
    fn period(
        #[case] expect: Result<Duration, AUTDInternalError>,
        #[case] f: Freq<f32>,
        #[case] n: usize,
    ) {
        assert_eq!(
            expect,
            GainSTM::new(f, (0..n).map(|_| Null::default())).map(|f| f.period())
        );
    }

    #[rstest::rstest]
    #[test]
    #[case::phase_intensity_full(GainSTMMode::PhaseIntensityFull)]
    #[case::phase_full(GainSTMMode::PhaseFull)]
    #[case::phase_half(GainSTMMode::PhaseHalf)]
    #[cfg_attr(miri, ignore)]
    fn with_mode(#[case] mode: GainSTMMode) {
        assert_eq!(
            mode,
            GainSTM::new(SamplingConfig::FREQ_40K, (0..2).map(|_| Null::default()))
                .unwrap()
                .with_mode(mode)
                .mode()
        );
    }

    #[rstest::rstest]
    #[test]
    #[cfg_attr(miri, ignore)]
    fn with_mode_default() {
        assert_eq!(
            GainSTMMode::PhaseIntensityFull,
            GainSTM::new(SamplingConfig::FREQ_40K, (0..2).map(|_| Null::default()))
                .unwrap()
                .mode()
        );
    }

    #[rstest::rstest]
    #[test]
    #[case::infinite(LoopBehavior::infinite())]
    #[case::finite(LoopBehavior::once())]
    #[cfg_attr(miri, ignore)]
    fn with_loop_behavior(#[case] loop_behavior: LoopBehavior) {
        assert_eq!(
            loop_behavior,
            GainSTM::new(SamplingConfig::FREQ_40K, (0..2).map(|_| Null::default()))
                .unwrap()
                .with_loop_behavior(loop_behavior)
                .loop_behavior()
        );
    }
}
