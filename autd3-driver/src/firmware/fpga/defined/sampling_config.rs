use std::{fmt::Debug, time::Duration};

use crate::{
    error::AUTDInternalError,
    firmware::fpga::{FPGA_CLK_FREQ, SAMPLING_FREQ_DIV_MAX, SAMPLING_FREQ_DIV_MIN},
};

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Frequency(f64);
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Period(Duration);
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Division(u32);

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum SamplingConfiguration {
    Frequency(Frequency),
    Period(Period),
    Division(Division),
}

impl SamplingConfiguration {
    pub const BASE_FREQUENCY: u32 = FPGA_CLK_FREQ;
    pub const DISABLE: Self = Self::Division(Division(0xFFFFFFFF));
    pub const FREQ_4K_HZ: Self = Self::Frequency(Frequency(4e3));
}

impl SamplingConfiguration {
    pub const DIV_MIN_RAW: u32 = SAMPLING_FREQ_DIV_MIN;
    pub const DIV_MAX_RAW: u32 = SAMPLING_FREQ_DIV_MAX;
    pub const DIV_MIN: u32 = SAMPLING_FREQ_DIV_MIN;
    pub const DIV_MAX: u32 =
        (SAMPLING_FREQ_DIV_MAX / SAMPLING_FREQ_DIV_MIN) * SAMPLING_FREQ_DIV_MIN;

    pub const FREQ_MIN_RAW: f64 = Self::BASE_FREQUENCY as f64 / Self::DIV_MAX_RAW as f64;
    pub const FREQ_MAX_RAW: f64 = Self::BASE_FREQUENCY as f64 / Self::DIV_MIN_RAW as f64;
    pub const FREQ_MIN: u32 = 1;
    pub const FREQ_MAX: u32 = Self::BASE_FREQUENCY / Self::DIV_MIN;

    pub const PERIOD_MIN_RAW: Duration =
        Duration::from_nanos((1000000000. / Self::FREQ_MAX_RAW as f64) as u64);
    pub const PERIOD_MAX_RAW: Duration = Duration::from_nanos(209715196875);
    pub const PERIOD_MIN: Duration = Duration::from_nanos(1000000000 / Self::FREQ_MAX as u64);
    pub const PERIOD_MAX: Duration = Duration::from_nanos(209715175000);

    pub fn from_division(div: u32) -> Result<Self, AUTDInternalError> {
        if div % Self::DIV_MIN != 0 {
            Err(AUTDInternalError::SamplingFreqDivInvalid(div))
        } else {
            Self::from_division_raw(div)
        }
    }

    pub fn from_division_raw(div: u32) -> Result<Self, AUTDInternalError> {
        if !(Self::DIV_MIN_RAW..=Self::DIV_MAX_RAW).contains(&div) {
            Err(AUTDInternalError::SamplingFreqDivOutOfRange(
                div,
                Self::DIV_MIN_RAW,
                Self::DIV_MAX_RAW,
            ))
        } else {
            Ok(Self::Division(Division(div)))
        }
    }

    pub fn from_frequency(f: u32) -> Result<Self, AUTDInternalError> {
        if (super::ULTRASOUND_FREQUENCY % f) != 0 {
            Err(AUTDInternalError::SamplingFreqInvalid(
                f,
                super::ULTRASOUND_FREQUENCY,
            ))
        } else {
            Self::from_frequency_nearest(f as _)
        }
    }

    pub fn from_frequency_nearest(f: f64) -> Result<Self, AUTDInternalError> {
        if !(Self::FREQ_MIN_RAW..=Self::FREQ_MAX_RAW).contains(&f) {
            Err(AUTDInternalError::SamplingFreqOutOfRange(
                f,
                Self::FREQ_MIN_RAW,
                Self::FREQ_MAX_RAW,
            ))
        } else {
            Ok(Self::Frequency(Frequency(f)))
        }
    }

    pub fn from_period(p: std::time::Duration) -> Result<Self, AUTDInternalError> {
        if p.as_nanos() % Self::PERIOD_MIN.as_nanos() != 0 {
            return Err(AUTDInternalError::SamplingPeriodInvalid(
                p,
                Self::PERIOD_MIN,
            ));
        }
        Self::from_period_nearest(p)
    }

    pub fn from_period_nearest(p: std::time::Duration) -> Result<Self, AUTDInternalError> {
        if !(Self::PERIOD_MIN_RAW..=Self::PERIOD_MAX_RAW).contains(&p) {
            Err(AUTDInternalError::SamplingPeriodOutOfRange(
                p,
                Self::PERIOD_MIN_RAW,
                Self::PERIOD_MAX_RAW,
            ))
        } else {
            Ok(Self::Period(Period(p)))
        }
    }

    pub fn division(&self) -> u32 {
        match self {
            Self::Frequency(f) => (Self::BASE_FREQUENCY as f64 / f.0) as _,
            Self::Period(p) => {
                (Self::BASE_FREQUENCY as f64 * (p.0.as_nanos() as f64 / 1000000000.)) as _
            }
            Self::Division(d) => d.0,
        }
    }

    pub fn frequency(&self) -> f64 {
        match self {
            Self::Frequency(f) => f.0,
            Self::Period(p) => 1000000000. / p.0.as_nanos() as f64,
            Self::Division(d) => Self::BASE_FREQUENCY as f64 / d.0 as f64,
        }
    }

    pub fn period(&self) -> Duration {
        match self {
            Self::Frequency(f) => Duration::from_nanos((1000000000. / f.0) as _),
            Self::Period(p) => p.0,
            Self::Division(d) => {
                Duration::from_nanos((1000000000. / Self::BASE_FREQUENCY as f64 * d.0 as f64) as _)
            }
        }
    }
}

impl std::fmt::Display for SamplingConfiguration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SamplingConfiguration::Frequency(freq) => write!(f, "Frequency({})", freq.0),
            SamplingConfiguration::Period(p) => write!(f, "Period({:?})", p.0),
            SamplingConfiguration::Division(d) => write!(f, "Division({})", d.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::firmware::fpga::ULTRASOUND_FREQUENCY;

    use super::*;

    #[rstest::rstest]
    #[test]
    #[case::min(
        Ok(SamplingConfiguration::Division(Division(SamplingConfiguration::DIV_MIN))),
        SamplingConfiguration::DIV_MIN
    )]
    #[case::max(
        Ok(SamplingConfiguration::Division(Division(SamplingConfiguration::DIV_MAX))),
        SamplingConfiguration::DIV_MAX
    )]
    #[case::invalid(
        Err(AUTDInternalError::SamplingFreqDivInvalid(
            SamplingConfiguration::DIV_MIN + 1
        )),
        SamplingConfiguration::DIV_MIN + 1
    )]
    #[case::out_of_range(
        Err(AUTDInternalError::SamplingFreqDivOutOfRange(
            0,
            SamplingConfiguration::DIV_MIN_RAW,
            SamplingConfiguration::DIV_MAX_RAW
        )),
        0
    )]
    fn from_division(
        #[case] expected: Result<SamplingConfiguration, AUTDInternalError>,
        #[case] freq_div: u32,
    ) {
        assert_eq!(expected, SamplingConfiguration::from_division(freq_div));
    }

    #[rstest::rstest]
    #[test]
    #[case::min(
        Ok(SamplingConfiguration::Division(Division(SamplingConfiguration::DIV_MIN_RAW))),
        SamplingConfiguration::DIV_MIN_RAW
    )]
    #[case::max(
        Ok(SamplingConfiguration::Division(Division(SamplingConfiguration::DIV_MAX_RAW))),
        SamplingConfiguration::DIV_MAX_RAW
    )]
    #[case::invalid(
        Ok(SamplingConfiguration::Division(Division(SamplingConfiguration::DIV_MIN_RAW + 1))),
        SamplingConfiguration::DIV_MIN_RAW + 1
    )]
    #[case::out_of_range(
        Err(AUTDInternalError::SamplingFreqDivOutOfRange(
            0,
            SamplingConfiguration::DIV_MIN_RAW,
            SamplingConfiguration::DIV_MAX_RAW
        )),
        0
    )]
    fn from_division_raw(
        #[case] expected: Result<SamplingConfiguration, AUTDInternalError>,
        #[case] freq_div: u32,
    ) {
        assert_eq!(expected, SamplingConfiguration::from_division_raw(freq_div));
    }

    #[rstest::rstest]
    #[test]
    #[case::min(
        Ok(SamplingConfiguration::Frequency(Frequency(SamplingConfiguration::FREQ_MIN as _))),
        SamplingConfiguration::FREQ_MIN
    )]
    #[case::max(
        Ok(SamplingConfiguration::Frequency(Frequency(SamplingConfiguration::FREQ_MAX as _))),
        SamplingConfiguration::FREQ_MAX
    )]
    #[case(
        Err(AUTDInternalError::SamplingFreqInvalid(512, ULTRASOUND_FREQUENCY)),
        512
    )]
    #[case::not_supported_max(
        Err(AUTDInternalError::SamplingFreqInvalid(
            SamplingConfiguration::FREQ_MAX - 1,
            ULTRASOUND_FREQUENCY
        )),
        SamplingConfiguration::FREQ_MAX - 1
    )]
    #[case::out_of_range_max(
        Err(AUTDInternalError::SamplingFreqInvalid(
            SamplingConfiguration::FREQ_MAX * 2,
            ULTRASOUND_FREQUENCY
        )),
        SamplingConfiguration::FREQ_MAX * 2
    )]
    fn from_frequency(
        #[case] expected: Result<SamplingConfiguration, AUTDInternalError>,
        #[case] freq: u32,
    ) {
        assert_eq!(expected, SamplingConfiguration::from_frequency(freq));
    }

    #[rstest::rstest]
    #[test]
    #[case::min(
        Ok(SamplingConfiguration::Frequency(Frequency(SamplingConfiguration::FREQ_MIN_RAW))),
        SamplingConfiguration::FREQ_MIN_RAW
    )]
    #[case::max(
        Ok(SamplingConfiguration::Frequency(Frequency(SamplingConfiguration::FREQ_MAX_RAW))),
        SamplingConfiguration::FREQ_MAX_RAW
    )]
    #[case(Ok(SamplingConfiguration::Frequency(Frequency(512.))), 512.)]
    #[case::not_supported_max(
        Ok(SamplingConfiguration::Frequency(Frequency(SamplingConfiguration::FREQ_MAX as f64 - 1.))),
        SamplingConfiguration::FREQ_MAX as f64 - 1.
    )]
    #[case::out_of_range_min(
        Err(AUTDInternalError::SamplingFreqOutOfRange(
            SamplingConfiguration::FREQ_MIN_RAW as f64 - f64::MIN,
            SamplingConfiguration::FREQ_MIN_RAW,
            SamplingConfiguration::FREQ_MAX_RAW
        )),
        SamplingConfiguration::FREQ_MIN_RAW as f64 - f64::MIN
    )]
    #[case::out_of_range_max(
        Err(AUTDInternalError::SamplingFreqOutOfRange(
            SamplingConfiguration::FREQ_MAX_RAW as f64 + f64::MIN,
            SamplingConfiguration::FREQ_MIN_RAW,
            SamplingConfiguration::FREQ_MAX_RAW
        )),
        SamplingConfiguration::FREQ_MAX_RAW as f64 + f64::MIN
    )]
    fn from_frequency_nearest(
        #[case] expected: Result<SamplingConfiguration, AUTDInternalError>,
        #[case] freq: f64,
    ) {
        assert_eq!(
            expected,
            SamplingConfiguration::from_frequency_nearest(freq)
        );
    }

    #[rstest::rstest]
    #[test]
    #[case::min(
        Ok(SamplingConfiguration::Period(Period(SamplingConfiguration::PERIOD_MIN))),
        SamplingConfiguration::PERIOD_MIN
    )]
    #[case::max(
        Ok(SamplingConfiguration::Period(Period(SamplingConfiguration::PERIOD_MAX))),
        SamplingConfiguration::PERIOD_MAX
    )]
    #[case::not_supported_min(
        Err(AUTDInternalError::SamplingPeriodInvalid(
            SamplingConfiguration::PERIOD_MIN + Duration::from_nanos(1),
            SamplingConfiguration::PERIOD_MIN

        )),
        SamplingConfiguration::PERIOD_MIN + Duration::from_nanos(1)
    )]
    #[case::not_supported_max(
        Err(AUTDInternalError::SamplingPeriodInvalid(
            SamplingConfiguration::PERIOD_MAX - Duration::from_nanos(1),
            SamplingConfiguration::PERIOD_MIN
        )),
        SamplingConfiguration::PERIOD_MAX - Duration::from_nanos(1)
    )]
    #[case::out_of_range_min(
        Err(AUTDInternalError::SamplingPeriodInvalid(
            SamplingConfiguration::PERIOD_MIN / 2,
            SamplingConfiguration::PERIOD_MIN
        )),
        SamplingConfiguration::PERIOD_MIN / 2
    )]
    #[case::out_of_range_max(
        Err(AUTDInternalError::SamplingPeriodOutOfRange(
            SamplingConfiguration::PERIOD_MAX * 2,
            SamplingConfiguration::PERIOD_MIN_RAW,
            SamplingConfiguration::PERIOD_MAX_RAW
        )),
        SamplingConfiguration::PERIOD_MAX * 2
    )]
    fn from_period(
        #[case] expected: Result<SamplingConfiguration, AUTDInternalError>,
        #[case] period: Duration,
    ) {
        assert_eq!(expected, SamplingConfiguration::from_period(period));
    }

    #[rstest::rstest]
    #[test]
    #[case::min(
        Ok(SamplingConfiguration::Period(Period(SamplingConfiguration::PERIOD_MIN_RAW))),
        SamplingConfiguration::PERIOD_MIN_RAW
    )]
    #[case::max(
        Ok(SamplingConfiguration::Period(Period(SamplingConfiguration::PERIOD_MAX_RAW))),
        SamplingConfiguration::PERIOD_MAX_RAW
    )]
    #[case::not_supported_min(
        Ok(SamplingConfiguration::Period(Period(SamplingConfiguration::PERIOD_MIN_RAW + Duration::from_nanos(1)))),
        SamplingConfiguration::PERIOD_MIN_RAW + Duration::from_nanos(1)
    )]
    #[case::not_supported_max(
        Ok(SamplingConfiguration::Period(Period(SamplingConfiguration::PERIOD_MAX_RAW - Duration::from_nanos(1)))),
        SamplingConfiguration::PERIOD_MAX_RAW - Duration::from_nanos(1)
    )]
    #[case::out_of_range_min(
        Err(AUTDInternalError::SamplingPeriodOutOfRange(
            SamplingConfiguration::PERIOD_MIN_RAW - Duration::from_nanos(1),
            SamplingConfiguration::PERIOD_MIN_RAW,
            SamplingConfiguration::PERIOD_MAX_RAW
        )),
        SamplingConfiguration::PERIOD_MIN_RAW - Duration::from_nanos(1)
    )]
    #[case::out_of_range_max(
        Err(AUTDInternalError::SamplingPeriodOutOfRange(
            SamplingConfiguration::PERIOD_MAX_RAW + Duration::from_nanos(1),
            SamplingConfiguration::PERIOD_MIN_RAW,
            SamplingConfiguration::PERIOD_MAX_RAW
        )),
        SamplingConfiguration::PERIOD_MAX_RAW + Duration::from_nanos(1)
    )]
    fn from_period_nearest(
        #[case] expected: Result<SamplingConfiguration, AUTDInternalError>,
        #[case] period: Duration,
    ) {
        assert_eq!(expected, SamplingConfiguration::from_period_nearest(period));
    }
}
