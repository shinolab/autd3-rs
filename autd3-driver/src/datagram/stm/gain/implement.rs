use std::{collections::HashMap, iter::Peekable};

use autd3_core::{
    derive::DatagramOption,
    gain::{BitVec, Gain, GainContext, GainContextGenerator, GainError},
};

use crate::geometry::{Device, Geometry};

use super::{GainSTMContext, GainSTMContextGenerator, GainSTMGenerator};

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

impl<G: Gain> GainSTMGenerator for Vec<G> {
    type T = Vec<G::G>;

    fn init(
        self,
        geometry: &Geometry,
        filter: Option<&HashMap<usize, BitVec>>,
        option: &DatagramOption,
    ) -> Result<Self::T, GainError> {
        self.into_iter()
            .map(|g| g.init_full(geometry, filter, option))
            .collect::<Result<Vec<_>, _>>()
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

    use super::super::GainSTM;
    use crate::{
        datagram::{gain::tests::TestGain, GainSTMOption},
        defined::{kHz, Freq, Hz},
        error::AUTDDriverError,
        firmware::fpga::SamplingConfig,
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
            GainSTM {
                gains: (0..n).map(|_| TestGain::null()).collect::<Vec<_>>(),
                config: freq,
                option: GainSTMOption::default()
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
            GainSTM {
                gains: (0..n).map(|_| TestGain::null()).collect::<Vec<_>>(),
                config: freq,
                option: GainSTMOption::default()
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
            GainSTM {
                gains: (0..n).map(|_| TestGain::null()).collect::<Vec<_>>(),
                config: p,
                option: GainSTMOption::default()
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
            GainSTM {
                gains: (0..n).map(|_| TestGain::null()).collect::<Vec<_>>(),
                config: p,
                option: GainSTMOption::default()
            }
            .into_nearest()
            .sampling_config()
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
            GainSTM {
                gains: (0..n).map(|_| TestGain::null()).collect::<Vec<_>>(),
                config,
                option: GainSTMOption::default()
            }
            .sampling_config()
        );
        Ok(())
    }
}
