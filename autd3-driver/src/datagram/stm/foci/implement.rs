use std::sync::Arc;

use crate::{error::AUTDDriverError, geometry::Device};

use super::{ControlPoints, FociSTMContext, FociSTMContextGenerator, FociSTMGenerator};

pub struct VecFociSTMContext<const N: usize, C>
where
    C: Send + Sync,
    ControlPoints<N>: From<C>,
{
    foci: Arc<Vec<C>>,
    i: usize,
}

impl<const N: usize, C> FociSTMContext<N> for VecFociSTMContext<N, C>
where
    C: Clone + Send + Sync,
    ControlPoints<N>: From<C>,
{
    fn next(&mut self) -> ControlPoints<N> {
        let p = self.foci[self.i].clone().into();
        self.i += 1;
        p
    }
}

impl<const N: usize, C> FociSTMContextGenerator<N> for Arc<Vec<C>>
where
    C: Clone + Send + Sync + std::fmt::Debug,
    ControlPoints<N>: From<C>,
{
    type Context = VecFociSTMContext<N, C>;

    fn generate(&mut self, _: &Device) -> Self::Context {
        Self::Context {
            foci: self.clone(),
            i: 0,
        }
    }
}

impl<const N: usize, C> FociSTMGenerator<N> for Vec<C>
where
    C: Clone + Send + Sync + std::fmt::Debug,
    ControlPoints<N>: From<C>,
{
    type T = Arc<Vec<C>>;

    fn init(self) -> Result<Self::T, AUTDDriverError> {
        Ok(Arc::new(self))
    }

    fn len(&self) -> usize {
        self.len()
    }
}

#[cfg(test)]
mod tests {
    #[cfg(not(feature = "dynamic_freq"))]
    use std::time::Duration;

    use autd3_core::modulation::SamplingConfigError;

    use super::{super::FociSTM, *};
    use crate::{
        defined::{kHz, Freq, Hz},
        firmware::fpga::SamplingConfig,
        geometry::Point3,
    };

    #[rstest::rstest]
    #[test]
    #[case((1. * Hz).try_into(), 0.5*Hz, 2)]
    #[case((10. * Hz).try_into(), 1.*Hz, 10)]
    #[case((20. * Hz).try_into(), 2.*Hz, 10)]
    #[case((2. * 0.49*Hz).try_into(), 0.49*Hz, 2)]
    fn from_freq(
        #[case] expect: Result<SamplingConfig, SamplingConfigError>,
        #[case] freq: Freq<f32>,
        #[case] n: usize,
    ) {
        assert_eq!(
            expect.map_err(AUTDDriverError::from),
            FociSTM {
                foci: (0..n).map(|_| Point3::origin()).collect::<Vec<_>>(),
                config: freq,
            }
            .sampling_config()
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
            Ok(expect),
            FociSTM {
                foci: (0..n).map(|_| Point3::origin()).collect::<Vec<_>>(),
                config: freq,
            }
            .into_nearest()
            .sampling_config()
        );
    }

    #[cfg(not(feature = "dynamic_freq"))]
    #[rstest::rstest]
    #[test]
    #[case(
        Duration::from_millis(1000).try_into().map_err(AUTDDriverError::from),
        Duration::from_millis(2000),
        2
    )]
    #[case(
        Duration::from_millis(100).try_into().map_err(AUTDDriverError::from),
        Duration::from_millis(1000),
        10
    )]
    #[case(
        Duration::from_millis(50).try_into().map_err(AUTDDriverError::from),
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
            FociSTM {
                foci: (0..n).map(|_| Point3::origin()).collect::<Vec<_>>(),
                config: p,
            }
            .sampling_config()
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
            Ok(expect),
            FociSTM {
                foci: (0..n).map(|_| Point3::origin()).collect::<Vec<_>>(),
                config: p,
            }
            .into_nearest()
            .sampling_config()
        );
    }

    #[rstest::rstest]
    #[test]
    #[case((4. * kHz).try_into().unwrap(), 10)]
    #[case((8. * kHz).try_into().unwrap(), 10)]
    fn from_sampling_config(#[case] config: SamplingConfig, #[case] n: usize) {
        assert_eq!(
            Ok(config),
            FociSTM {
                foci: (0..n).map(|_| Point3::origin()).collect::<Vec<_>>(),
                config,
            }
            .sampling_config()
        );
    }
}
