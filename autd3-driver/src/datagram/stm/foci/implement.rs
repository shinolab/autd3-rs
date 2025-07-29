use std::{borrow::Borrow, sync::Arc};

use crate::{error::AUTDDriverError, geometry::Device};

use super::{ControlPoints, FociSTMGenerator, FociSTMIterator, FociSTMIteratorGenerator};

pub struct VecFociSTMIterator<const N: usize, C, I>
where
    ControlPoints<N>: From<C>,
{
    foci: Arc<I>,
    i: usize,
    _phantom: std::marker::PhantomData<C>,
}

impl<const N: usize, C, I> FociSTMIterator<N> for VecFociSTMIterator<N, C, I>
where
    I: Borrow<[C]> + Send + Sync,
    C: Clone + Send + Sync,
    ControlPoints<N>: From<C>,
{
    fn next(&mut self) -> ControlPoints<N> {
        let p = <I as Borrow<[C]>>::borrow(&self.foci)[self.i]
            .clone()
            .into();
        self.i += 1;
        p
    }
}

impl<const N: usize, C, I> FociSTMIteratorGenerator<N> for VecFociSTMIterator<N, C, I>
where
    I: Borrow<[C]> + Send + Sync,
    C: Clone + Send + Sync,
    ControlPoints<N>: From<C>,
{
    type Iterator = VecFociSTMIterator<N, C, I>;

    fn generate(&mut self, _: &Device) -> Self::Iterator {
        Self::Iterator {
            foci: self.foci.clone(),
            i: 0,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<const N: usize, C> FociSTMGenerator<N> for Vec<C>
where
    C: Clone + Send + Sync,
    ControlPoints<N>: From<C>,
{
    type T = VecFociSTMIterator<N, C, Vec<C>>;

    fn init(self) -> Result<Self::T, AUTDDriverError> {
        Ok(VecFociSTMIterator {
            foci: Arc::new(self),
            i: 0,
            _phantom: std::marker::PhantomData,
        })
    }

    fn len(&self) -> usize {
        Vec::len(self)
    }
}

impl<const M: usize, const N: usize, C> FociSTMGenerator<N> for [C; M]
where
    C: Clone + Send + Sync,
    ControlPoints<N>: From<C>,
{
    type T = VecFociSTMIterator<N, C, [C; M]>;

    fn init(self) -> Result<Self::T, AUTDDriverError> {
        Ok(VecFociSTMIterator {
            foci: Arc::new(self),
            i: 0,
            _phantom: std::marker::PhantomData,
        })
    }

    fn len(&self) -> usize {
        M
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::super::FociSTM;
    use crate::{
        common::{Freq, Hz, kHz},
        geometry::Point3,
    };

    use autd3_core::firmware::SamplingConfig;

    #[rstest::rstest]
    #[case(SamplingConfig::new(1. * Hz), 0.5*Hz, 2)]
    #[case(SamplingConfig::new(10. * Hz), 1.*Hz, 10)]
    #[case(SamplingConfig::new(20. * Hz), 2.*Hz, 10)]
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

    #[rstest::rstest]
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

    #[rstest::rstest]
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
