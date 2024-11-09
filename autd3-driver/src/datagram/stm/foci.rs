use super::sampling_config::*;
use crate::{
    datagram::*,
    defined::{ControlPoints, Freq},
    derive::*,
    firmware::operation::FociSTMOp,
};

pub use crate::firmware::operation::FociSTMContext;

use derive_more::{Deref, DerefMut};
use silencer::WithSampling;

#[allow(clippy::len_without_is_empty)]
pub trait FociSTMContextGenerator<const N: usize>: std::fmt::Debug {
    type Context: FociSTMContext<N>;
    fn generate(&mut self, device: &Device) -> Self::Context;
}

pub struct VecFociSTMContext<const N: usize> {
    foci: Arc<Vec<ControlPoints<N>>>,
    i: usize,
}

impl<const N: usize> FociSTMContext<N> for VecFociSTMContext<N> {
    fn next(&mut self) -> ControlPoints<N> {
        let p = self.foci[self.i].clone();
        self.i += 1;
        p
    }
}

impl<const N: usize> FociSTMContextGenerator<N> for Arc<Vec<ControlPoints<N>>> {
    type Context = VecFociSTMContext<N>;

    fn generate(&mut self, _: &Device) -> Self::Context {
        Self::Context {
            foci: self.clone(),
            i: 0,
        }
    }
}

#[allow(clippy::len_without_is_empty)]
pub trait FociSTMGenerator<const N: usize>: std::fmt::Debug {
    type T: FociSTMContextGenerator<N>;

    fn init(self) -> Result<Self::T, AUTDInternalError>;
    fn len(&self) -> usize;
}

impl<const N: usize> FociSTMGenerator<N> for Vec<ControlPoints<N>> {
    type T = Arc<Vec<ControlPoints<N>>>;

    fn init(self) -> Result<Self::T, AUTDInternalError> {
        Ok(Arc::new(self))
    }

    fn len(&self) -> usize {
        self.len()
    }
}

pub trait IntoFociSTMGenerator<const N: usize> {
    type G: FociSTMGenerator<N>;

    fn into(self) -> Self::G;
}

impl<const N: usize, C, T> IntoFociSTMGenerator<N> for T
where
    T: IntoIterator<Item = C>,
    ControlPoints<N>: From<C>,
{
    type G = Vec<ControlPoints<N>>;

    fn into(self) -> Self::G {
        self.into_iter().map(ControlPoints::from).collect()
    }
}

#[derive(Clone, Builder, Deref, DerefMut, Debug)]
pub struct FociSTM<const N: usize, G: FociSTMGenerator<N>> {
    #[deref]
    #[deref_mut]
    gen: G,
    #[get]
    #[set]
    loop_behavior: LoopBehavior,
    #[get]
    sampling_config: SamplingConfig,
    size: usize,
}

impl<const N: usize, G: FociSTMGenerator<N>> WithSampling for FociSTM<N, G> {
    fn sampling_config_intensity(&self) -> Option<SamplingConfig> {
        Some(self.sampling_config)
    }

    fn sampling_config_phase(&self) -> Option<SamplingConfig> {
        Some(self.sampling_config)
    }
}

impl<const N: usize, G: FociSTMGenerator<N>> FociSTM<N, G> {
    pub fn new(
        config: impl Into<STMConfig>,
        iter: impl IntoFociSTMGenerator<N, G = G>,
    ) -> Result<Self, AUTDInternalError> {
        Self::new_from_sampling_config(config.into(), iter)
    }

    pub fn new_nearest(
        config: impl Into<STMConfigNearest>,
        iter: impl IntoFociSTMGenerator<N, G = G>,
    ) -> Result<Self, AUTDInternalError> {
        Self::new_from_sampling_config(config.into(), iter)
    }

    fn new_from_sampling_config<T>(
        config: T,
        iter: impl IntoFociSTMGenerator<N, G = G>,
    ) -> Result<Self, AUTDInternalError>
    where
        SamplingConfig: TryFrom<(T, usize), Error = AUTDInternalError>,
    {
        let gen = iter.into();
        let size = gen.len();
        Ok(Self {
            sampling_config: (config, size).try_into()?,
            gen,
            loop_behavior: LoopBehavior::infinite(),
            size,
        })
    }

    pub fn freq(&self) -> Freq<f32> {
        self.sampling_config().freq() / self.gen.len() as f32
    }

    pub fn period(&self) -> Duration {
        self.sampling_config().period() * self.gen.len() as u32
    }
}

pub struct FociSTMOperationGenerator<const N: usize, G: FociSTMContextGenerator<N>> {
    gen: G,
    size: usize,
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
    segment: Segment,
    transition_mode: Option<TransitionMode>,
}

impl<const N: usize, G: FociSTMContextGenerator<N>> OperationGenerator
    for FociSTMOperationGenerator<N, G>
{
    type O1 = FociSTMOp<N, G::Context>;
    type O2 = NullOp;

    fn generate(&mut self, device: &Device) -> (Self::O1, Self::O2) {
        (
            Self::O1::new(
                self.gen.generate(device),
                self.size,
                self.config,
                self.loop_behavior,
                self.segment,
                self.transition_mode,
            ),
            Self::O2::new(),
        )
    }
}

impl<const N: usize, G: FociSTMGenerator<N>> DatagramS for FociSTM<N, G> {
    type G = FociSTMOperationGenerator<N, G::T>;

    fn operation_generator_with_segment(
        self,
        _geometry: &Geometry,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> Result<Self::G, AUTDInternalError> {
        Ok(FociSTMOperationGenerator {
            gen: self.gen.init()?,
            size: self.size,
            config: self.sampling_config,
            loop_behavior: self.loop_behavior,
            segment,
            transition_mode,
        })
    }

    fn parallel_threshold(&self) -> Option<usize> {
        if self.gen.len() * N >= 4000 {
            None
        } else {
            Some(usize::MAX)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        defined::{kHz, Hz},
        geometry::Vector3,
    };

    use super::*;

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
            FociSTM::new(freq, (0..n).map(|_| Vector3::zeros())).map(|f| f.sampling_config())
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
            FociSTM::new_nearest(freq, (0..n).map(|_| Vector3::zeros()))
                .map(|f| f.sampling_config())
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
            FociSTM::new(p, (0..n).map(|_| Vector3::zeros())).map(|f| f.sampling_config())
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
            FociSTM::new_nearest(p, (0..n).map(|_| Vector3::zeros())).map(|f| f.sampling_config())
        );
    }

    #[rstest::rstest]
    #[test]
    #[case((4. * kHz).try_into().unwrap(), 10)]
    #[case((8. * kHz).try_into().unwrap(), 10)]
    fn from_sampling_config(#[case] config: SamplingConfig, #[case] n: usize) {
        assert_eq!(
            Ok(config),
            FociSTM::new(config, (0..n).map(|_| Vector3::zeros())).map(|f| f.sampling_config())
        );
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
            FociSTM::new(f, (0..n).map(|_| Vector3::zeros())).map(|f| f.freq())
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
            FociSTM::new(f, (0..n).map(|_| Vector3::zeros())).map(|f| f.period())
        );
    }

    #[rstest::rstest]
    #[test]
    #[case::infinite(LoopBehavior::infinite())]
    #[case::finite(LoopBehavior::once())]
    fn with_loop_behavior(#[case] loop_behavior: LoopBehavior) -> anyhow::Result<()> {
        assert_eq!(
            loop_behavior,
            FociSTM::new(1. * Hz, (0..2).map(|_| Vector3::zeros()))?
                .with_loop_behavior(loop_behavior)
                .loop_behavior()
        );
        Ok(())
    }

    #[test]
    fn with_loop_behavior_deafault() -> anyhow::Result<()> {
        let stm = FociSTM::new(1. * Hz, (0..2).map(|_| Vector3::zeros()))?;
        assert_eq!(LoopBehavior::infinite(), stm.loop_behavior());
        Ok(())
    }
}
