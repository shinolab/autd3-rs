use std::iter::Peekable;

use super::sampling_config::*;
use crate::{
    datagram::*,
    defined::Freq,
    derive::*,
    firmware::{cpu::GainSTMMode, operation::GainSTMOp},
};

pub use crate::firmware::operation::GainSTMContext;

use derive_more::{Deref, DerefMut};
use silencer::WithSampling;

pub trait GainSTMContextGenerator {
    type Gain: GainContextGenerator;
    type Context: GainSTMContext<Context = <Self::Gain as GainContextGenerator>::Context>;

    fn generate(&mut self, device: &Device) -> Self::Context;
}

pub struct VecGainSTMContext<G: GainContext> {
    gains: Peekable<std::vec::IntoIter<G>>,
}

impl<G: GainContext> GainSTMContext for VecGainSTMContext<G> {
    type Context = G;

    fn next(&mut self) -> Option<Self::Context> {
        self.gains.next()
    }
}

impl<G: GainContextGenerator> GainSTMContextGenerator for Vec<G> {
    type Gain = G;
    type Context = VecGainSTMContext<G::Context>;

    fn generate(&mut self, device: &Device) -> Self::Context {
        Self::Context {
            gains: self
                .iter_mut()
                .map(|g| g.generate(device))
                .collect::<Vec<_>>()
                .into_iter()
                .peekable(),
        }
    }
}

#[allow(clippy::len_without_is_empty)]
pub trait GainSTMGenerator: std::fmt::Debug {
    type T: GainSTMContextGenerator;

    fn init(
        self,
        geometry: &Geometry,
        filter: Option<&HashMap<usize, BitVec<u32>>>,
    ) -> Result<Self::T, AUTDInternalError>;
    fn len(&self) -> usize;
}

impl<G: Gain> GainSTMGenerator for Vec<G> {
    type T = Vec<G::G>;

    fn init(
        self,
        geometry: &Geometry,
        filter: Option<&HashMap<usize, BitVec<u32>>>,
    ) -> Result<Self::T, AUTDInternalError> {
        self.into_iter()
            .map(|g| g.init(geometry, filter))
            .collect::<Result<Vec<_>, _>>()
    }
    fn len(&self) -> usize {
        self.len()
    }
}

pub trait IntoGainSTMGenerator {
    type G: GainSTMGenerator;

    fn into(self) -> Self::G;
}

impl<G: Gain, T> IntoGainSTMGenerator for T
where
    T: IntoIterator<Item = G>,
{
    type G = Vec<G>;

    fn into(self) -> Self::G {
        self.into_iter().collect()
    }
}

#[derive(Builder, Clone, Debug, Deref, DerefMut)]
pub struct GainSTM<G: GainSTMGenerator> {
    #[deref]
    #[deref_mut]
    gen: G,
    #[get]
    #[set]
    loop_behavior: LoopBehavior,
    #[get]
    sampling_config: SamplingConfig,
    #[get]
    #[set]
    mode: GainSTMMode,
}

impl<G: GainSTMGenerator> WithSampling for GainSTM<G> {
    fn sampling_config_intensity(&self) -> Option<SamplingConfig> {
        Some(self.sampling_config)
    }

    fn sampling_config_phase(&self) -> Option<SamplingConfig> {
        Some(self.sampling_config)
    }
}

impl<G: GainSTMGenerator> GainSTM<G> {
    pub fn new<T: IntoGainSTMGenerator<G = G>>(
        config: impl Into<STMConfig>,
        iter: T,
    ) -> Result<Self, AUTDInternalError> {
        Self::new_from_sampling_config(config.into(), iter)
    }

    pub fn new_nearest<T: IntoGainSTMGenerator<G = G>>(
        config: impl Into<STMConfigNearest>,
        iter: T,
    ) -> Result<Self, AUTDInternalError> {
        Self::new_from_sampling_config(config.into(), iter)
    }

    fn new_from_sampling_config<S, T: IntoGainSTMGenerator<G = G>>(
        config: S,
        iter: T,
    ) -> Result<Self, AUTDInternalError>
    where
        SamplingConfig: TryFrom<(S, usize), Error = AUTDInternalError>,
    {
        let gen = iter.into();
        Ok(Self {
            sampling_config: (config, gen.len()).try_into()?,
            loop_behavior: LoopBehavior::infinite(),
            mode: GainSTMMode::PhaseIntensityFull,
            gen,
        })
    }

    pub fn freq(&self) -> Freq<f32> {
        self.sampling_config().freq() / self.gen.len() as f32
    }

    pub fn period(&self) -> Duration {
        self.sampling_config().period() * self.gen.len() as u32
    }
}

pub struct GainSTMOperationGenerator<T: GainSTMContextGenerator> {
    g: T,
    size: usize,
    mode: GainSTMMode,
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
    segment: Segment,
    transition_mode: Option<TransitionMode>,
}

impl<T: GainSTMContextGenerator> OperationGenerator for GainSTMOperationGenerator<T> {
    type O1 = GainSTMOp<<T::Gain as GainContextGenerator>::Context, T::Context>;
    type O2 = NullOp;

    fn generate(&mut self, device: &Device) -> (Self::O1, Self::O2) {
        (
            Self::O1::new(
                self.g.generate(device),
                self.size,
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

impl<I: GainSTMGenerator> DatagramS for GainSTM<I> {
    type G = GainSTMOperationGenerator<I::T>;

    fn operation_generator_with_segment(
        self,
        geometry: &Geometry,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> Result<Self::G, AUTDInternalError> {
        let size = self.gen.len();
        let config = self.sampling_config;
        let loop_behavior = self.loop_behavior;
        let mode = self.mode;
        let initializer = self.gen;
        Ok(GainSTMOperationGenerator {
            g: initializer.init(geometry, None)?,
            size,
            config,
            mode,
            loop_behavior,
            segment,
            transition_mode,
        })
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
