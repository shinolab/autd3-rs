use std::time::Duration;

use crate::{
    defined::{Freq, Hz},
    error::AUTDInternalError,
    firmware::fpga::SamplingConfig,
    utils::float::is_integer,
};

#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum STMSamplingConfig {
    Freq(Freq<f32>),
    FreqNearest(Freq<f32>),
    Period(Duration),
    PeriodNearest(Duration),
    SamplingConfig(SamplingConfig),
}

impl STMSamplingConfig {
    pub fn sampling(&self, size: usize) -> Result<SamplingConfig, AUTDInternalError> {
        match *self {
            STMSamplingConfig::Freq(f) => {
                let fs = f.hz() as f64 * size as f64;
                if !is_integer(fs) {
                    return Err(AUTDInternalError::STMFreqInvalid(size, f));
                }
                Ok(SamplingConfig::Freq(fs as u32 * Hz))
            }
            STMSamplingConfig::FreqNearest(f) => {
                Ok(SamplingConfig::FreqNearest(f.hz() * size as f32 * Hz))
            }
            Self::SamplingConfig(s) => Ok(s),
            Self::Period(p) => {
                if p.as_nanos() % size as u128 != 0 {
                    return Err(AUTDInternalError::STMPeriodInvalid(size, p));
                }
                Ok(SamplingConfig::Period(p / size as u32))
            }
            Self::PeriodNearest(p) => Ok(SamplingConfig::PeriodNearest(p / size as u32)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{defined::Hz, derive::AUTDInternalError, firmware::fpga::SamplingConfig};

    #[rstest::rstest]
    #[test]
    #[case(Ok(SamplingConfig::Freq(4000*Hz)), 4000.*Hz, 1)]
    #[case(Ok(SamplingConfig::Freq(8000*Hz)), 4000.*Hz, 2)]
    #[case(Ok(SamplingConfig::Freq(40000*Hz)), 40000.*Hz, 1)]
    #[case(Err(AUTDInternalError::STMFreqInvalid(1, 4000.5*Hz)), 4000.5*Hz, 1)]
    fn frequency(
        #[case] expect: Result<SamplingConfig, AUTDInternalError>,
        #[case] freq: Freq<f32>,
        #[case] size: usize,
    ) {
        assert_eq!(expect, STMSamplingConfig::Freq(freq).sampling(size));
    }

    #[rstest::rstest]
    #[test]
    #[case(Ok(SamplingConfig::FreqNearest(4000.*Hz)), 4000.*Hz, 1)]
    #[case(Ok(SamplingConfig::FreqNearest(8000.*Hz)), 4000.*Hz, 2)]
    #[case(Ok(SamplingConfig::FreqNearest(4001.*Hz)), 4001.*Hz, 1)]
    #[case(Ok(SamplingConfig::FreqNearest(40000.*Hz)), 40000.*Hz, 1)]
    fn frequency_nearest(
        #[case] expect: Result<SamplingConfig, AUTDInternalError>,
        #[case] freq: Freq<f32>,
        #[case] size: usize,
    ) {
        assert_eq!(expect, STMSamplingConfig::FreqNearest(freq).sampling(size));
    }

    #[rstest::rstest]
    #[test]
    #[case(SamplingConfig::FREQ_40K, 1)]
    #[case(SamplingConfig::FREQ_40K, 2)]
    #[case(SamplingConfig::FREQ_4K, 1)]
    #[case(SamplingConfig::FREQ_4K, 2)]
    fn sampling(#[case] config: SamplingConfig, #[case] size: usize) {
        assert_eq!(
            Ok(config),
            STMSamplingConfig::SamplingConfig(config).sampling(size)
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(
        Ok(SamplingConfig::Period(Duration::from_micros(250))),
        Duration::from_micros(250),
        1
    )]
    #[case(
        Ok(SamplingConfig::Period(Duration::from_micros(125))),
        Duration::from_micros(250),
        2
    )]
    #[case(
        Ok(SamplingConfig::Period(Duration::from_micros(25))),
        Duration::from_micros(25),
        1
    )]
    #[case(
        Err(AUTDInternalError::STMPeriodInvalid(2, Duration::from_nanos(25001))),
        Duration::from_nanos(25001),
        2
    )]
    fn period(
        #[case] expect: Result<SamplingConfig, AUTDInternalError>,
        #[case] p: Duration,
        #[case] size: usize,
    ) {
        assert_eq!(expect, STMSamplingConfig::Period(p).sampling(size));
    }

    #[rstest::rstest]
    #[test]
    #[case(
        Ok(SamplingConfig::PeriodNearest(Duration::from_micros(250))),
        Duration::from_micros(250),
        1
    )]
    #[case(
        Ok(SamplingConfig::PeriodNearest(Duration::from_micros(125))),
        Duration::from_micros(250),
        2
    )]
    #[case(
        Ok(SamplingConfig::PeriodNearest(Duration::from_micros(25))),
        Duration::from_micros(25),
        1
    )]
    #[case(
        Ok(SamplingConfig::PeriodNearest(Duration::from_nanos(12500))),
        Duration::from_nanos(25001),
        2
    )]
    fn period_nearest(
        #[case] expect: Result<SamplingConfig, AUTDInternalError>,
        #[case] p: Duration,
        #[case] size: usize,
    ) {
        assert_eq!(expect, STMSamplingConfig::PeriodNearest(p).sampling(size));
    }
}
