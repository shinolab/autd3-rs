use std::sync::Arc;

use crate::{error::AUTDDriverError, geometry::Device};

use super::{
    ControlPoints, FociSTMContext, FociSTMContextGenerator, FociSTMGenerator, IntoFociSTMGenerator,
};

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

impl<const N: usize> FociSTMGenerator<N> for Vec<ControlPoints<N>> {
    type T = Arc<Vec<ControlPoints<N>>>;

    fn init(self) -> Result<Self::T, AUTDDriverError> {
        Ok(Arc::new(self))
    }

    fn len(&self) -> usize {
        self.len()
    }
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

#[cfg(test)]
mod tests {
    #[cfg(not(feature = "dynamic_freq"))]
    use std::time::Duration;

    use super::{super::FociSTM, *};
    use crate::{
        defined::{kHz, Freq, Hz},
        firmware::fpga::{LoopBehavior, SamplingConfig},
        geometry::Point3,
    };

    #[rstest::rstest]
    #[test]
    #[case((1. * Hz).try_into(), 0.5*Hz, 2)]
    #[case((10. * Hz).try_into(), 1.*Hz, 10)]
    #[case((20. * Hz).try_into(), 2.*Hz, 10)]
    #[case((2. * 0.49*Hz).try_into(), 0.49*Hz, 2)]
    fn from_freq(
        #[case] expect: Result<SamplingConfig, AUTDDriverError>,
        #[case] freq: Freq<f32>,
        #[case] n: usize,
    ) {
        assert_eq!(
            expect,
            FociSTM::new(freq, (0..n).map(|_| Point3::origin())).map(|f| f.sampling_config())
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(SamplingConfig::new_nearest(1. * Hz), 0.5*Hz, 2)]
    #[case(SamplingConfig::new_nearest(0.98 * Hz), 0.49*Hz, 2)]
    #[case(SamplingConfig::new_nearest(10. * Hz), 1.*Hz, 10)]
    #[case(SamplingConfig::new_nearest(20. * Hz), 2.*Hz, 10)]
    fn from_freq_nearest(
        #[case] expect: SamplingConfig,
        #[case] freq: Freq<f32>,
        #[case] n: usize,
    ) {
        assert_eq!(
            expect,
            FociSTM::new_nearest(freq, (0..n).map(|_| Point3::origin())).sampling_config()
        );
    }

    #[cfg(not(feature = "dynamic_freq"))]
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
    #[case(Err(AUTDDriverError::STMPeriodInvalid(2, Duration::from_millis(2000) + Duration::from_nanos(1))), Duration::from_millis(2000) + Duration::from_nanos(1), 2)]
    fn from_period(
        #[case] expect: Result<SamplingConfig, AUTDDriverError>,
        #[case] p: Duration,
        #[case] n: usize,
    ) {
        assert_eq!(
            expect,
            FociSTM::new(p, (0..n).map(|_| Point3::origin())).map(|f| f.sampling_config())
        );
    }

    #[cfg(not(feature = "dynamic_freq"))]
    #[rstest::rstest]
    #[test]
    #[case(
        Duration::from_millis(1000).try_into().unwrap(),
        Duration::from_millis(2000),
        2
    )]
    #[case(
        Duration::from_millis(100).try_into().unwrap(),
        Duration::from_millis(1000),
        10
    )]
    #[case(
        Duration::from_millis(50).try_into().unwrap(),
        Duration::from_millis(500),
        10
    )]
    #[case(Duration::from_millis(1000).try_into().unwrap(), Duration::from_millis(2000) + Duration::from_nanos(1), 2)]
    fn from_period_nearest(#[case] expect: SamplingConfig, #[case] p: Duration, #[case] n: usize) {
        assert_eq!(
            expect,
            FociSTM::new_nearest(p, (0..n).map(|_| Point3::origin())).sampling_config()
        );
    }

    #[rstest::rstest]
    #[test]
    #[case((4. * kHz).try_into().unwrap(), 10)]
    #[case((8. * kHz).try_into().unwrap(), 10)]
    fn from_sampling_config(#[case] config: SamplingConfig, #[case] n: usize) {
        assert_eq!(
            Ok(config),
            FociSTM::new(config, (0..n).map(|_| Point3::origin())).map(|f| f.sampling_config())
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(Ok(0.5*Hz), 0.5*Hz, 2)]
    #[case(Ok(1.0*Hz), 1.*Hz, 10)]
    #[case(Ok(2.0*Hz), 2.*Hz, 10)]
    fn freq(
        #[case] expect: Result<Freq<f32>, AUTDDriverError>,
        #[case] f: Freq<f32>,
        #[case] n: usize,
    ) {
        assert_eq!(
            expect,
            FociSTM::new(f, (0..n).map(|_| Point3::origin())).map(|f| f.freq())
        );
    }

    #[cfg(not(feature = "dynamic_freq"))]
    #[rstest::rstest]
    #[test]
    #[case(Ok(Duration::from_millis(2000)), 0.5*Hz, 2)]
    #[case(Ok(Duration::from_millis(1000)), 1.*Hz, 10)]
    #[case(Ok(Duration::from_millis(500)), 2.*Hz, 10)]
    fn period(
        #[case] expect: Result<Duration, AUTDDriverError>,
        #[case] f: Freq<f32>,
        #[case] n: usize,
    ) {
        assert_eq!(
            expect,
            FociSTM::new(f, (0..n).map(|_| Point3::origin())).map(|f| f.period())
        );
    }

    #[rstest::rstest]
    #[test]
    #[case::infinite(LoopBehavior::infinite())]
    #[case::finite(LoopBehavior::once())]
    fn with_loop_behavior(#[case] loop_behavior: LoopBehavior) -> anyhow::Result<()> {
        assert_eq!(
            loop_behavior,
            FociSTM::new(1. * Hz, (0..2).map(|_| Point3::origin()))?
                .with_loop_behavior(loop_behavior)
                .loop_behavior()
        );
        Ok(())
    }

    #[test]
    fn with_loop_behavior_deafault() -> anyhow::Result<()> {
        let stm = FociSTM::new(1. * Hz, (0..2).map(|_| Point3::origin()))?;
        assert_eq!(LoopBehavior::infinite(), stm.loop_behavior());
        Ok(())
    }
}
