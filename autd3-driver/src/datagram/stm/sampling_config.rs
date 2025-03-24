use std::time::Duration;

use crate::{defined::Freq, error::AUTDDriverError, firmware::fpga::SamplingConfig};

#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
/// Sampling configuration for STM.
pub enum STMConfig {
    #[doc(hidden)]
    Freq(Freq<f32>),

    #[doc(hidden)]
    Period(Duration),
    #[doc(hidden)]
    SamplingConfig(SamplingConfig),
    #[doc(hidden)]
    FreqNearest(Freq<f32>),

    #[doc(hidden)]
    PeriodNearest(Duration),
}

impl STMConfig {
    // must be public for capi
    #[doc(hidden)]
    pub fn into_sampling_config(self, size: usize) -> Result<SamplingConfig, AUTDDriverError> {
        match self {
            STMConfig::Freq(f) => Ok(SamplingConfig::new(f * size as f32)),

            STMConfig::Period(p) => {
                if p.as_nanos() % size as u128 != 0 {
                    return Err(AUTDDriverError::STMPeriodInvalid(size, p));
                }
                Ok(SamplingConfig::new(p / size as u32))
            }
            STMConfig::SamplingConfig(s) => Ok(s),
            STMConfig::FreqNearest(freq) => {
                Ok(SamplingConfig::new(freq * size as f32).into_nearest())
            }

            STMConfig::PeriodNearest(duration) => {
                Ok(SamplingConfig::new(duration / size as u32).into_nearest())
            }
        }
    }
}

impl From<Freq<f32>> for STMConfig {
    fn from(freq: Freq<f32>) -> Self {
        Self::Freq(freq)
    }
}

impl From<Duration> for STMConfig {
    fn from(p: Duration) -> Self {
        Self::Period(p)
    }
}

impl From<SamplingConfig> for STMConfig {
    fn from(config: SamplingConfig) -> Self {
        Self::SamplingConfig(config)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FreqNearest(pub Freq<f32>);
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PeriodNearest(pub Duration);

impl From<FreqNearest> for STMConfig {
    fn from(f: FreqNearest) -> Self {
        STMConfig::FreqNearest(f.0)
    }
}

impl From<PeriodNearest> for STMConfig {
    fn from(p: PeriodNearest) -> Self {
        STMConfig::PeriodNearest(p.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{defined::Hz, firmware::fpga::SamplingConfig};

    #[rstest::rstest]
    #[test]
    #[case(SamplingConfig::new(4000. * Hz), 4000. * Hz, 1)]
    #[case(SamplingConfig::new(8000. * Hz,), 4000. * Hz, 2)]
    #[case(SamplingConfig::new(40000. * Hz), 40000. * Hz, 1)]
    fn frequency(#[case] expect: SamplingConfig, #[case] freq: Freq<f32>, #[case] size: usize) {
        assert_eq!(Ok(expect), STMConfig::Freq(freq).into_sampling_config(size));
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
            STMConfig::SamplingConfig(config).into_sampling_config(size)
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(
        Ok(SamplingConfig::new(Duration::from_micros(250))),
        Duration::from_micros(250),
        1
    )]
    #[case(
        Ok(SamplingConfig::new(Duration::from_micros(125))),
        Duration::from_micros(250),
        2
    )]
    #[case(
        Ok(SamplingConfig::new(Duration::from_micros(25))),
        Duration::from_micros(25),
        1
    )]
    #[case(
        Err(AUTDDriverError::STMPeriodInvalid(2, Duration::from_nanos(25001))),
        Duration::from_nanos(25001),
        2
    )]
    fn period(
        #[case] expect: Result<SamplingConfig, AUTDDriverError>,
        #[case] p: Duration,
        #[case] size: usize,
    ) {
        assert_eq!(expect, STMConfig::Period(p).into_sampling_config(size));
    }

    #[rstest::rstest]
    #[test]
    #[case(SamplingConfig::new(4000. * Hz).into_nearest(), 4000. * Hz, 1)]
    #[case(SamplingConfig::new(8000. * Hz).into_nearest(), 4000. * Hz, 2)]
    #[case(SamplingConfig::new(4001. * Hz).into_nearest(), 4001. * Hz, 1)]
    #[case(SamplingConfig::new(40000. * Hz).into_nearest(), 40000. * Hz, 1)]
    fn frequency_nearest(
        #[case] expect: SamplingConfig,
        #[case] freq: Freq<f32>,
        #[case] size: usize,
    ) {
        assert_eq!(
            Ok(expect),
            STMConfig::FreqNearest(freq).into_sampling_config(size)
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(
        SamplingConfig::new(Duration::from_micros(250)).into_nearest(),
        Duration::from_micros(250),
        1
    )]
    #[case(
        SamplingConfig::new(Duration::from_micros(125)).into_nearest(),
        Duration::from_micros(250),
        2
    )]
    #[case(
        SamplingConfig::new(Duration::from_micros(25)).into_nearest(),
        Duration::from_micros(25),
        1
    )]
    #[case(
        SamplingConfig::new(Duration::from_nanos(12500)).into_nearest(),
        Duration::from_nanos(25001),
        2
    )]
    fn period_nearest(#[case] expect: SamplingConfig, #[case] p: Duration, #[case] size: usize) {
        assert_eq!(
            Ok(expect),
            STMConfig::PeriodNearest(p).into_sampling_config(size)
        );
    }
}
