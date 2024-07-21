use std::{fmt::Debug, num::NonZeroU16, time::Duration};

use crate::{
    defined::{Freq, Hz, ULTRASOUND_FREQ},
    error::AUTDInternalError,
};

use derive_more::Display;

const NANOSEC: u128 = 1_000_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Display)]
#[non_exhaustive]
pub enum SamplingConfig {
    #[display(fmt = "{}", _0)]
    Freq(Freq<u32>),
    #[display(fmt = "{}", _0)]
    FreqNearest(Freq<f32>),
    #[display(fmt = "{:?}", _0)]
    Period(Duration),
    #[display(fmt = "{:?}", _0)]
    PeriodNearest(Duration),
    #[display(fmt = "Division({})", _0)]
    Division(NonZeroU16),
}

const FREQ_MIN: Freq<f32> = Freq {
    freq: ULTRASOUND_FREQ.hz() as f32 / u16::MAX as f32,
};
const FREQ_MAX: Freq<f32> = Freq {
    freq: ULTRASOUND_FREQ.hz() as f32,
};
const PERIOD_MIN: Duration = Duration::from_nanos((NANOSEC / ULTRASOUND_FREQ.hz() as u128) as u64);
const PERIOD_MAX: Duration =
    Duration::from_nanos((u16::MAX as u128 * NANOSEC / ULTRASOUND_FREQ.hz() as u128) as u64);

impl SamplingConfig {
    pub const FREQ_40K: SamplingConfig = SamplingConfig::Freq(Freq { freq: 40000 }) ;
    pub const FREQ_4K: SamplingConfig = SamplingConfig::Freq(Freq { freq: 4000 }) ;

    fn division_from_freq_nearest(f: Freq<f32>) -> Result<u16, AUTDInternalError> {
        if !(FREQ_MIN..=FREQ_MAX).contains(&f) {
            Err(AUTDInternalError::SamplingFreqOutOfRange(
                f, FREQ_MIN, FREQ_MAX,
            ))
        } else {
            Ok((ULTRASOUND_FREQ.hz() as f32 / f.hz()) as _)
        }
    }

    fn division_from_period_nearest(p: Duration) -> Result<u16, AUTDInternalError> {
        if !(PERIOD_MIN..=PERIOD_MAX).contains(&p) {
            Err(AUTDInternalError::SamplingPeriodOutOfRange(
                p, PERIOD_MIN, PERIOD_MAX,
            ))
        } else {
            let k = (p.as_nanos() * ULTRASOUND_FREQ.hz() as u128) / NANOSEC;
            Ok(k as _)
        }
    }

    pub fn division(&self) -> Result<u16, AUTDInternalError> {
        match *self {
            Self::Division(div) => Ok(div.get()),
            Self::Freq(f) => {
                if ULTRASOUND_FREQ.hz() % f.hz() != 0 {
                    return Err(AUTDInternalError::SamplingFreqInvalid(f, ULTRASOUND_FREQ));
                }
                Self::division_from_freq_nearest((f.hz() as f32) * Hz)
            }
            Self::FreqNearest(f) => Self::division_from_freq_nearest(f),
            Self::Period(p) => {
                let k = p.as_nanos() * ULTRASOUND_FREQ.hz() as u128;
                if k % NANOSEC != 0 {
                    return Err(AUTDInternalError::SamplingPeriodInvalid(p));
                }
                Self::division_from_period_nearest(p)
            }
            Self::PeriodNearest(p) => Self::division_from_period_nearest(p),
        }
    }

    pub fn freq(&self) -> Result<Freq<f32>, AUTDInternalError> {
        self.division()
            .map(|d| ULTRASOUND_FREQ.hz() as f32 / d as f32 * Hz)
    }

    pub fn period(&self) -> Result<Duration, AUTDInternalError> {
        self.division().map(|d| {
            Duration::from_nanos((d as u128 * NANOSEC / ULTRASOUND_FREQ.hz() as u128) as u64)
        })
    }
}

impl From<Freq<u32>> for SamplingConfig {
    fn from(freq: Freq<u32>) -> Self {
        Self::Freq(freq)
    }
}

impl From<Duration> for SamplingConfig {
    fn from(p: Duration) -> Self {
        Self::Period(p)
    }
}

#[cfg(test)]
mod tests {
    use crate::defined::Hz;

    use super::*;

    #[rstest::rstest]
    #[test]
    #[case::min(Ok(1), NonZeroU16::new(1).unwrap())]
    #[case::max(Ok(u16::MAX), NonZeroU16::new(u16::MAX).unwrap())]
    #[cfg_attr(miri, ignore)]
    fn division_from_division(
        #[case] expected: Result<u16, AUTDInternalError>,
        #[case] div: NonZeroU16,
    ) {
        assert_eq!(expected, SamplingConfig::Division(div).division());
    }

    #[rstest::rstest]
    #[test]
    #[case(Ok(Duration::from_micros(25)), SamplingConfig::Division(NonZeroU16::new(1).unwrap()))]
    #[case(Ok(Duration::from_micros(25)), SamplingConfig::Freq(40000*Hz))]
    #[case(
        Ok(Duration::from_micros(25)),
        SamplingConfig::Period(Duration::from_micros(25))
    )]
    #[cfg_attr(miri, ignore)]
    fn period(
        #[case] expected: Result<Duration, AUTDInternalError>,
        #[case] config: SamplingConfig,
    ) {
        assert_eq!(expected, config.period());
    }

    #[rstest::rstest]
    #[test]
    #[case::min(Ok(40000), 1 * Hz)]
    #[case::max(Ok(1), ULTRASOUND_FREQ)]
    #[case(Err(AUTDInternalError::SamplingFreqInvalid(512*Hz, 40000 * Hz)), 512 * Hz)]
    #[case::not_supported_max(
        Err(AUTDInternalError::SamplingFreqInvalid(
            ULTRASOUND_FREQ - 1 * Hz,
            40000 * Hz
        )),
        ULTRASOUND_FREQ - 1 * Hz,
    )]
    #[case::out_of_range_max(
        Err(AUTDInternalError::SamplingFreqInvalid(
            ULTRASOUND_FREQ * 2,
            40000 * Hz
        )),
        ULTRASOUND_FREQ * 2,
    )]
    #[cfg_attr(miri, ignore)]
    fn from_freq(#[case] expected: Result<u16, AUTDInternalError>, #[case] freq: Freq<u32>) {
        assert_eq!(expected, SamplingConfig::Freq(freq).division());
    }

    #[rstest::rstest]
    #[test]
    #[case::min(Ok(0xFFFF), FREQ_MIN)]
    #[case::max(Ok(1), FREQ_MAX)]
    #[case::not_supported_max(
        Ok(1),
        (ULTRASOUND_FREQ.hz() as f32 - 1.)*Hz,
    )]
    #[case::out_of_range_min(
        Err(AUTDInternalError::SamplingFreqOutOfRange(
            FREQ_MIN - f32::MIN * Hz,
            FREQ_MIN,
            FREQ_MAX
        )),
        FREQ_MIN - f32::MIN*Hz,
    )]
    #[case::out_of_range_max(
        Err(AUTDInternalError::SamplingFreqOutOfRange(
            FREQ_MAX + f32::MIN * Hz,
            FREQ_MIN,
            FREQ_MAX
        )),
        FREQ_MAX + f32::MIN*Hz,
    )]
    #[cfg_attr(miri, ignore)]
    fn from_freq_nearest(
        #[case] expected: Result<u16, AUTDInternalError>,
        #[case] freq: Freq<f32>,
    ) {
        assert_eq!(expected, SamplingConfig::FreqNearest(freq).division());
    }

    #[rstest::rstest]
    #[test]
    #[case::min(Ok(1), PERIOD_MIN)]
    #[case::max(Ok(65535), PERIOD_MAX)]
    #[case(
        Err(AUTDInternalError::SamplingPeriodInvalid(Duration::from_micros(26))),
        Duration::from_micros(26)
    )]
    #[case::not_supported_max(
        Err(AUTDInternalError::SamplingPeriodInvalid(
            PERIOD_MAX - Duration::from_nanos(1) 
        )),
        PERIOD_MAX - Duration::from_nanos(1),
    )]
    #[case::out_of_range_max(
        Err(AUTDInternalError::SamplingPeriodOutOfRange(
            PERIOD_MAX * 2,
            PERIOD_MIN,
            PERIOD_MAX
        )),
        PERIOD_MAX * 2,
    )]
    #[cfg_attr(miri, ignore)]
    fn from_period(#[case] expected: Result<u16, AUTDInternalError>, #[case] period: Duration) {
        assert_eq!(expected, SamplingConfig::from(period).division());
    }

    #[rstest::rstest]
    #[test]
    #[case::min(Ok(1), PERIOD_MIN)]
    #[case::max(Ok(65535), PERIOD_MAX)]
    #[case(Ok(1), Duration::from_micros(26))]
    #[case::not_supported_max(
        Ok(65534),
        PERIOD_MAX - Duration::from_nanos(1),
    )]
    #[case::out_of_range_min(
        Err(AUTDInternalError::SamplingPeriodOutOfRange(
            PERIOD_MIN - Duration::from_nanos(1),
            PERIOD_MIN,
            PERIOD_MAX
        )),
        PERIOD_MIN - Duration::from_nanos(1),
    )]
    #[case::out_of_range_max(
        Err(AUTDInternalError::SamplingPeriodOutOfRange(
            PERIOD_MAX * 2,
            PERIOD_MIN,
            PERIOD_MAX
        )),
        PERIOD_MAX * 2,
    )]
    #[cfg_attr(miri, ignore)]
    fn from_period_nearest(
        #[case] expected: Result<u16, AUTDInternalError>,
        #[case] period: Duration,
    ) {
        assert_eq!(expected, SamplingConfig::PeriodNearest(period).division());
    }

    #[rstest::rstest]
    #[test]
    #[case::freq(SamplingConfig::Freq(4000*Hz), "4000 Hz")]
    #[case::freq(SamplingConfig::FreqNearest(4000.*Hz), "4000 Hz")]
    #[case::div(SamplingConfig::Division(NonZeroU16::new(12345).unwrap()), "Division(12345)")]
    #[case::div(SamplingConfig::Period(Duration::from_micros(25)), "25µs")]
    #[case::div(SamplingConfig::PeriodNearest(Duration::from_micros(25)), "25µs")]
    #[cfg_attr(miri, ignore)]
    fn display(#[case] config: SamplingConfig, #[case] expected: &str) {
        assert_eq!(expected, config.to_string());
    }
}
