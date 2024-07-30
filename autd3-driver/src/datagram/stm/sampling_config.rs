use std::time::Duration;

use crate::{
    defined::{Freq, Hz},
    error::AUTDInternalError,
    firmware::fpga::{IntoSamplingConfig, IntoSamplingConfigNearest, SamplingConfig},
};

#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum STMConfig {
    Freq(Freq<f32>),
    Period(Duration),
    SamplingConfig(SamplingConfig),
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum STMConfigNearest {
    Freq(Freq<f32>),
    Period(Duration),
}

impl TryFrom<(STMConfig, usize)> for SamplingConfig {
    type Error = AUTDInternalError;

    fn try_from(value: (STMConfig, usize)) -> Result<Self, Self::Error> {
        let (config, size) = value;
        match config {
            STMConfig::Freq(f) => (f * size as f32).into_sampling_config(),
            STMConfig::Period(p) => {
                if p.as_nanos() % size as u128 != 0 {
                    return Err(AUTDInternalError::STMPeriodInvalid(size, p));
                }
                (p / size as u32).into_sampling_config()
            }
            STMConfig::SamplingConfig(s) => Ok(s),
        }
    }
}

impl TryFrom<(STMConfigNearest, usize)> for SamplingConfig {
    type Error = AUTDInternalError;

    fn try_from(value: (STMConfigNearest, usize)) -> Result<Self, Self::Error> {
        let (config, size) = value;
        match config {
            STMConfigNearest::Freq(f) => {
                Ok((f.hz() * size as f32 * Hz).into_sampling_config_nearest())
            }
            STMConfigNearest::Period(p) => Ok((p / size as u32).into_sampling_config_nearest()),
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

impl From<Freq<f32>> for STMConfigNearest {
    fn from(freq: Freq<f32>) -> Self {
        Self::Freq(freq)
    }
}

impl From<Duration> for STMConfigNearest {
    fn from(p: Duration) -> Self {
        Self::Period(p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{defined::Hz, derive::AUTDInternalError, firmware::fpga::SamplingConfig};

    #[rstest::rstest]
    #[test]
    #[case((4000. * Hz).into_sampling_config(), 4000. * Hz, 1)]
    #[case((8000. * Hz).into_sampling_config(), 4000. * Hz, 2)]
    #[case((40000. * Hz).into_sampling_config(), 40000. * Hz, 1)]
    #[case((4000.5 * Hz).into_sampling_config(), 4000.5 * Hz, 1)]
    #[cfg_attr(miri, ignore)]
    fn frequency(
        #[case] expect: Result<SamplingConfig, AUTDInternalError>,
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
    #[cfg_attr(miri, ignore)]
    fn sampling(#[case] config: SamplingConfig, #[case] size: usize) {
        assert_eq!(
            Ok(config),
            (STMConfig::SamplingConfig(config), size).try_into()
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(
        Duration::from_micros(250).into_sampling_config(),
        Duration::from_micros(250),
        1
    )]
    #[case(
        Duration::from_micros(125).into_sampling_config(),
        Duration::from_micros(250),
        2
    )]
    #[case(
        Duration::from_micros(25).into_sampling_config(),
        Duration::from_micros(25),
        1
    )]
    #[case(
        Err(AUTDInternalError::STMPeriodInvalid(2, Duration::from_nanos(25001))),
        Duration::from_nanos(25001),
        2
    )]
    #[cfg_attr(miri, ignore)]
    fn period(
        #[case] expect: Result<SamplingConfig, AUTDInternalError>,
        #[case] p: Duration,
        #[case] size: usize,
    ) {
        assert_eq!(expect, (STMConfig::Period(p), size).try_into());
    }

    #[rstest::rstest]
    #[test]
    #[case(Ok((4000. * Hz).into_sampling_config_nearest()), 4000. * Hz, 1)]
    #[case(Ok((8000. * Hz).into_sampling_config_nearest()), 4000. * Hz, 2)]
    #[case(Ok((4001. * Hz).into_sampling_config_nearest()), 4001. * Hz, 1)]
    #[case(Ok((40000. * Hz).into_sampling_config_nearest()), 40000. * Hz, 1)]
    #[cfg_attr(miri, ignore)]
    fn frequency_nearest(
        #[case] expect: Result<SamplingConfig, AUTDInternalError>,
        #[case] freq: Freq<f32>,
        #[case] size: usize,
    ) {
        assert_eq!(expect, (STMConfigNearest::Freq(freq), size).try_into());
    }

    #[rstest::rstest]
    #[test]
    #[case(
        Ok(Duration::from_micros(250).into_sampling_config_nearest()),
        Duration::from_micros(250),
        1
    )]
    #[case(
        Ok(Duration::from_micros(125).into_sampling_config_nearest()),
        Duration::from_micros(250),
        2
    )]
    #[case(
        Ok(Duration::from_micros(25).into_sampling_config_nearest()),
        Duration::from_micros(25),
        1
    )]
    #[case(
        Ok(Duration::from_nanos(12500).into_sampling_config_nearest()),
        Duration::from_nanos(25001),
        2
    )]
    #[cfg_attr(miri, ignore)]
    fn period_nearest(
        #[case] expect: Result<SamplingConfig, AUTDInternalError>,
        #[case] p: Duration,
        #[case] size: usize,
    ) {
        assert_eq!(expect, (STMConfigNearest::Period(p), size).try_into());
    }
}
