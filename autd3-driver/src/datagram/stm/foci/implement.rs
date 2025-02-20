use std::sync::Arc;

use crate::{error::AUTDDriverError, geometry::Device};

use super::{ControlPoints, FociSTMGenerator, FociSTMIterator, FociSTMIteratorGenerator};

pub struct VecFociSTMIterator<const N: usize, C>
where
    C: Send + Sync,
    ControlPoints<N>: From<C>,
{
    foci: Arc<Vec<C>>,
    i: usize,
}

impl<const N: usize, C> FociSTMIterator<N> for VecFociSTMIterator<N, C>
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

impl<const N: usize, C> FociSTMIteratorGenerator<N> for Arc<Vec<C>>
where
    C: Clone + Send + Sync + std::fmt::Debug,
    ControlPoints<N>: From<C>,
{
    type Iterator = VecFociSTMIterator<N, C>;

    fn generate(&mut self, _: &Device) -> Self::Iterator {
        Self::Iterator {
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

    use super::super::FociSTM;
    use crate::{
        defined::{Freq, Hz, kHz},
        firmware::fpga::SamplingConfig,
        geometry::Point3,
    };

    #[rstest::rstest]
    #[test]
    #[case(SamplingConfig::new(1. * Hz), 0.5*Hz, 2)]
    #[case(SamplingConfig::new(10. * Hz), 1.*Hz, 10)]
    #[case(SamplingConfig::new(20. * Hz), 2.*Hz, 10)]
    #[case(SamplingConfig::new(2. * 0.49*Hz), 0.49*Hz, 2)]
    fn from_freq(#[case] expect: SamplingConfig, #[case] freq: Freq<f32>, #[case] n: usize) {
        assert_eq!(
            Ok(expect),
            FociSTM {
                foci: (0..n).map(|_| Point3::origin()).collect::<Vec<_>>(),
                config: freq,
            }
            .sampling_config()
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(SamplingConfig::new(1. * Hz).into_nearest(), 0.5*Hz, 2)]
    #[case(SamplingConfig::new(0.98 * Hz).into_nearest(), 0.49*Hz, 2)]
    #[case(SamplingConfig::new(10. * Hz).into_nearest(), 1.*Hz, 10)]
    #[case(SamplingConfig::new(20. * Hz).into_nearest(), 2.*Hz, 10)]
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
        Ok(SamplingConfig::new(Duration::from_millis(1000))),
        Duration::from_millis(2000),
        2
    )]
    #[case(
        Ok(SamplingConfig::new(Duration::from_millis(100))),
        Duration::from_millis(1000),
        10
    )]
    #[case(
        Ok(SamplingConfig::new(Duration::from_millis(50))),
        Duration::from_millis(500),
        10
    )]
    #[case(Err(crate::error::AUTDDriverError::STMPeriodInvalid(2, Duration::from_millis(2000) + Duration::from_nanos(1))), Duration::from_millis(2000) + Duration::from_nanos(1), 2)]
    fn from_period(
        #[case] expect: Result<SamplingConfig, crate::error::AUTDDriverError>,
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
        SamplingConfig::new(Duration::from_millis(1000)).into_nearest(),
        Duration::from_millis(2000),
        2
    )]
    #[case(
        SamplingConfig::new(Duration::from_millis(100)).into_nearest(),
        Duration::from_millis(1000),
        10
    )]
    #[case(
        SamplingConfig::new(Duration::from_millis(50)).into_nearest(),
        Duration::from_millis(500),
        10
    )]
    #[case(SamplingConfig::new(Duration::from_millis(1000)).into_nearest(), Duration::from_millis(2000) + Duration::from_nanos(1), 2)]
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
    #[case(SamplingConfig::new(4. * kHz), 10)]
    #[case(SamplingConfig::new(8. * kHz), 10)]
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
