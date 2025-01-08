#[cfg(not(feature = "dynamic_freq"))]
use std::time::Duration;

use crate::{
    defined::{Freq, Hz},
    error::AUTDDriverError,
    firmware::fpga::SamplingConfig,
};

#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
/// Sampling configuration for STM.
pub enum STMConfig {
    #[doc(hidden)]
    Freq(Freq<f32>),
    #[cfg(not(feature = "dynamic_freq"))]
    #[doc(hidden)]
    Period(Duration),
    #[doc(hidden)]
    SamplingConfig(SamplingConfig),
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// Sampling configuration for STM with nearest frequency or period.
#[non_exhaustive]
pub enum STMConfigNearest {
    #[doc(hidden)]
    Freq(Freq<f32>),
    #[cfg(not(feature = "dynamic_freq"))]
    #[doc(hidden)]
    Period(Duration),
}

impl TryFrom<(STMConfig, usize)> for SamplingConfig {
    type Error = AUTDDriverError;

    fn try_from(value: (STMConfig, usize)) -> Result<Self, Self::Error> {
        let (config, size) = value;
        match config {
            STMConfig::Freq(f) => SamplingConfig::new(f * size as f32),
            #[cfg(not(feature = "dynamic_freq"))]
            STMConfig::Period(p) => {
                if p.as_nanos() % size as u128 != 0 {
                    return Err(AUTDDriverError::STMPeriodInvalid(size, p));
                }
                SamplingConfig::new(p / size as u32)
            }
            STMConfig::SamplingConfig(s) => Ok(s),
        }
    }
}

impl TryFrom<(STMConfigNearest, usize)> for SamplingConfig {
    type Error = AUTDDriverError;

    fn try_from(value: (STMConfigNearest, usize)) -> Result<Self, Self::Error> {
        let (config, size) = value;
        match config {
            STMConfigNearest::Freq(f) => Ok(SamplingConfig::new_nearest(f.hz() * size as f32 * Hz)),
            #[cfg(not(feature = "dynamic_freq"))]
            STMConfigNearest::Period(p) => Ok(SamplingConfig::new_nearest(p / size as u32)),
        }
    }
}

impl From<Freq<f32>> for STMConfig {
    fn from(freq: Freq<f32>) -> Self {
        Self::Freq(freq)
    }
}

#[cfg(not(feature = "dynamic_freq"))]
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

impl From<Freq<f32>> for STMConfigNearest {
    fn from(freq: Freq<f32>) -> Self {
        Self::Freq(freq)
    }
}

#[cfg(not(feature = "dynamic_freq"))]
impl From<Duration> for STMConfigNearest {
    fn from(p: Duration) -> Self {
        Self::Period(p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{defined::Hz, firmware::fpga::SamplingConfig};

    #[rstest::rstest]
    #[test]
    #[case((4000. * Hz).try_into(), 4000. * Hz, 1)]
    #[case((8000. * Hz).try_into(), 4000. * Hz, 2)]
    #[case((40000. * Hz).try_into(), 40000. * Hz, 1)]
    #[case((4000.5 * Hz).try_into(), 4000.5 * Hz, 1)]
    fn frequency(
        #[case] expect: Result<SamplingConfig, AUTDDriverError>,
        #[case] freq: Freq<f32>,
        #[case] size: usize,
    ) {
        assert_eq!(expect, (STMConfig::Freq(freq), size).try_into());
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
            (STMConfig::SamplingConfig(config), size).try_into()
        );
    }

    #[cfg(not(feature = "dynamic_freq"))]
    #[rstest::rstest]
    #[test]
    #[case(
        Duration::from_micros(250).try_into(),
        Duration::from_micros(250),
        1
    )]
    #[case(
        Duration::from_micros(125).try_into(),
        Duration::from_micros(250),
        2
    )]
    #[case(
        Duration::from_micros(25).try_into(),
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
        assert_eq!(expect, (STMConfig::Period(p), size).try_into());
    }

    #[rstest::rstest]
    #[test]
    #[case(Ok(SamplingConfig::new_nearest(4000. * Hz)), 4000. * Hz, 1)]
    #[case(Ok(SamplingConfig::new_nearest(8000. * Hz)), 4000. * Hz, 2)]
    #[case(Ok(SamplingConfig::new_nearest(4001. * Hz)), 4001. * Hz, 1)]
    #[case(Ok(SamplingConfig::new_nearest(40000. * Hz)), 40000. * Hz, 1)]
    fn frequency_nearest(
        #[case] expect: Result<SamplingConfig, AUTDDriverError>,
        #[case] freq: Freq<f32>,
        #[case] size: usize,
    ) {
        assert_eq!(expect, (STMConfigNearest::Freq(freq), size).try_into());
    }

    #[cfg(not(feature = "dynamic_freq"))]
    #[rstest::rstest]
    #[test]
    #[case(
        Ok(SamplingConfig::new_nearest(Duration::from_micros(250))),
        Duration::from_micros(250),
        1
    )]
    #[case(
        Ok(SamplingConfig::new_nearest(Duration::from_micros(125))),
        Duration::from_micros(250),
        2
    )]
    #[case(
        Ok(SamplingConfig::new_nearest(Duration::from_micros(25))),
        Duration::from_micros(25),
        1
    )]
    #[case(
        Ok(SamplingConfig::new_nearest(Duration::from_nanos(12500))),
        Duration::from_nanos(25001),
        2
    )]
    fn period_nearest(
        #[case] expect: Result<SamplingConfig, AUTDDriverError>,
        #[case] p: Duration,
        #[case] size: usize,
    ) {
        assert_eq!(expect, (STMConfigNearest::Period(p), size).try_into());
    }
}
