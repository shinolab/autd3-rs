use std::iter::Peekable;

use autd3_core::gain::{
    Gain, GainCalculator, GainCalculatorGenerator, GainError, TransducerFilter,
};

use crate::geometry::{Device, Geometry};

use super::{GainSTMGenerator, GainSTMIterator, GainSTMIteratorGenerator};

pub struct VecGainSTMIterator<G: GainCalculator> {
    gains: Peekable<std::vec::IntoIter<G>>,
}

impl<G: GainCalculator> GainSTMIterator for VecGainSTMIterator<G> {
    type Calculator = G;

    fn next(&mut self) -> Option<Self::Calculator> {
        self.gains.next()
    }
}

impl<G: GainCalculatorGenerator> GainSTMIteratorGenerator for Vec<G> {
    type Gain = G;
    type Iterator = VecGainSTMIterator<G::Calculator>;

    fn generate(&mut self, device: &Device) -> Self::Iterator {
        Self::Iterator {
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

    fn init(self, geometry: &Geometry, filter: &TransducerFilter) -> Result<Self::T, GainError> {
        self.into_iter()
            .map(|g| g.init(geometry, filter))
            .collect::<Result<Vec<_>, _>>()
    }
    fn len(&self) -> usize {
        self.len()
    }
}

#[cfg(test)]
mod tests {

    use std::time::Duration;

    use super::super::GainSTM;
    use crate::{
        common::{Freq, Hz, kHz},
        datagram::{GainSTMOption, gain::tests::TestGain},
        firmware::fpga::SamplingConfig,
    };

    #[rstest::rstest]
    #[test]
    #[case(SamplingConfig::new(1. * Hz), 0.5*Hz, 2)]
    #[case(SamplingConfig::new(10. * Hz), 1.*Hz, 10)]
    #[case(SamplingConfig::new(20. * Hz), 2.*Hz, 10)]
    fn from_freq(#[case] expect: SamplingConfig, #[case] freq: Freq<f32>, #[case] n: usize) {
        assert_eq!(
            Ok(expect),
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
            GainSTM {
                gains: (0..n).map(|_| TestGain::null()).collect::<Vec<_>>(),
                config: freq,
                option: GainSTMOption::default()
            }
            .into_nearest()
            .sampling_config()
        );
    }

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
            GainSTM {
                gains: (0..n).map(|_| TestGain::null()).collect::<Vec<_>>(),
                config: p,
                option: GainSTMOption::default()
            }
            .sampling_config()
        );
    }

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
    #[case(SamplingConfig::new(4. * kHz), 10)]
    #[case(SamplingConfig::new(8. * kHz), 10)]
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
