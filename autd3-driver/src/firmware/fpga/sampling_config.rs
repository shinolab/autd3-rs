use std::{fmt::Debug, time::Duration};

use crate::{
    error::AUTDInternalError,
    firmware::fpga::{fpga_clk_freq, SAMPLING_FREQ_DIV_MAX, SAMPLING_FREQ_DIV_MIN},
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frequency(f64);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Period(Duration);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Division(u32);

#[derive(Clone, Copy, Debug)]
pub enum SamplingConfiguration {
    Frequency(Frequency),
    Period(Period),
    Division(Division),
}

#[const_fn::const_fn(cfg(not(feature = "variable_freq")))]
pub const fn base_frequency() -> u32 {
    fpga_clk_freq()
}

pub const fn div_min_raw() -> u32 {
    SAMPLING_FREQ_DIV_MIN
}
pub const fn div_max_raw() -> u32 {
    SAMPLING_FREQ_DIV_MAX
}
pub const fn div_min() -> u32 {
    SAMPLING_FREQ_DIV_MIN
}
pub const fn div_max() -> u32 {
    (SAMPLING_FREQ_DIV_MAX / SAMPLING_FREQ_DIV_MIN) * SAMPLING_FREQ_DIV_MIN
}

pub fn freq_min_raw() -> f64 {
    base_frequency() as f64 / div_max_raw() as f64
}
pub fn freq_max_raw() -> f64 {
    base_frequency() as f64 / div_min_raw() as f64
}
#[const_fn::const_fn(cfg(not(feature = "variable_freq")))]
pub const fn freq_min() -> u32 {
    1
}
#[const_fn::const_fn(cfg(not(feature = "variable_freq")))]
pub const fn freq_max() -> u32 {
    base_frequency() / div_min()
}
pub fn period_min_raw() -> Duration {
    Duration::from_nanos((1000000000. / freq_max_raw()) as u64)
}
#[const_fn::const_fn(cfg(not(feature = "variable_freq")))]
pub const fn period_max_raw() -> Duration {
    Duration::from_nanos(209715196875)
}
#[const_fn::const_fn(cfg(not(feature = "variable_freq")))]
pub const fn period_min() -> Duration {
    Duration::from_nanos(1000000000 / freq_max() as u64)
}
#[const_fn::const_fn(cfg(not(feature = "variable_freq")))]
pub const fn period_max() -> Duration {
    Duration::from_nanos(209715175000)
}

impl SamplingConfiguration {
    pub const DISABLE: Self = Self::Division(Division(0xFFFFFFFF));
    pub const FREQ_4K_HZ: Self = Self::Frequency(Frequency(4e3));

    pub fn from_division(div: u32) -> Result<Self, AUTDInternalError> {
        if div % div_min() != 0 {
            Err(AUTDInternalError::SamplingFreqDivInvalid(div))
        } else {
            Self::from_division_raw(div)
        }
    }

    pub fn from_division_raw(div: u32) -> Result<Self, AUTDInternalError> {
        if !(div_min_raw()..=div_max_raw()).contains(&div) {
            Err(AUTDInternalError::SamplingFreqDivOutOfRange(
                div,
                div_min_raw(),
                div_max_raw(),
            ))
        } else {
            Ok(Self::Division(Division(div)))
        }
    }

    pub fn from_freq(f: u32) -> Result<Self, AUTDInternalError> {
        if (super::ultrasound_freq() % f) != 0 {
            Err(AUTDInternalError::SamplingFreqInvalid(
                f,
                super::ultrasound_freq(),
            ))
        } else {
            Self::from_freq_nearest(f as _)
        }
    }

    pub fn from_freq_nearest(f: f64) -> Result<Self, AUTDInternalError> {
        if !(freq_min_raw()..=freq_max_raw()).contains(&f) {
            Err(AUTDInternalError::SamplingFreqOutOfRange(
                f,
                freq_min_raw(),
                freq_max_raw(),
            ))
        } else {
            Ok(Self::Frequency(Frequency(f)))
        }
    }

    pub fn from_period(p: std::time::Duration) -> Result<Self, AUTDInternalError> {
        if p.as_nanos() % period_min().as_nanos() != 0 {
            return Err(AUTDInternalError::SamplingPeriodInvalid(p, period_min()));
        }
        Self::from_period_nearest(p)
    }

    pub fn from_period_nearest(p: std::time::Duration) -> Result<Self, AUTDInternalError> {
        if !(period_min_raw()..=period_max_raw()).contains(&p) {
            Err(AUTDInternalError::SamplingPeriodOutOfRange(
                p,
                period_min_raw(),
                period_max_raw(),
            ))
        } else {
            Ok(Self::Period(Period(p)))
        }
    }

    pub fn division(&self) -> u32 {
        match self {
            Self::Frequency(f) => (base_frequency() as f64 / f.0) as _,
            Self::Period(p) => {
                (base_frequency() as f64 * (p.0.as_nanos() as f64 / 1000000000.)) as _
            }
            Self::Division(d) => d.0,
        }
    }

    pub fn freq(&self) -> f64 {
        base_frequency() as f64 / self.division() as f64
    }

    pub fn period(&self) -> Duration {
        Duration::from_nanos((1000000000. / base_frequency() as f64 * self.division() as f64) as _)
    }
}

impl std::fmt::Display for SamplingConfiguration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SamplingConfiguration::Frequency(freq) => write!(f, "{}Hz", freq.0),
            SamplingConfiguration::Period(p) => write!(f, "{:?}", p.0),
            SamplingConfiguration::Division(d) => write!(f, "Division({})", d.0),
        }
    }
}

impl std::cmp::PartialEq<SamplingConfiguration> for SamplingConfiguration {
    fn eq(&self, other: &SamplingConfiguration) -> bool {
        self.division().eq(&other.division())
    }
}

#[cfg(test)]
mod tests {
    use crate::firmware::fpga::{sampling_config, ULTRASOUND_FREQ};

    use super::*;

    #[rstest::rstest]
    #[test]
    #[case::min(
        Ok(SamplingConfiguration::Division(Division(sampling_config::div_min()))),
        sampling_config::div_min()
    )]
    #[case::max(
        Ok(SamplingConfiguration::Division(Division(sampling_config::div_max()))),
        sampling_config::div_max()
    )]
    #[case::invalid(
        Err(AUTDInternalError::SamplingFreqDivInvalid(
            sampling_config::div_min() + 1
        )),
        sampling_config::div_min() + 1
    )]
    #[case::out_of_range(
        Err(AUTDInternalError::SamplingFreqDivOutOfRange(
            0,
            sampling_config::div_min_raw(),
            sampling_config::div_max_raw()
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
        Ok(SamplingConfiguration::Division(Division(sampling_config::div_min_raw()))),
        sampling_config::div_min_raw()
    )]
    #[case::max(
        Ok(SamplingConfiguration::Division(Division(sampling_config::div_max_raw()))),
        sampling_config::div_max_raw()
    )]
    #[case::invalid(
        Ok(SamplingConfiguration::Division(Division(sampling_config::div_min_raw() + 1))),
        sampling_config::div_min_raw() + 1
    )]
    #[case::out_of_range(
        Err(AUTDInternalError::SamplingFreqDivOutOfRange(
            0,
            sampling_config::div_min_raw(),
            sampling_config::div_max_raw()
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
        Ok(SamplingConfiguration::Frequency(Frequency(sampling_config::freq_min() as _))),
        sampling_config::freq_min()
    )]
    #[case::max(
        Ok(SamplingConfiguration::Frequency(Frequency(sampling_config::freq_max() as _))),
        sampling_config::freq_max()
    )]
    #[case(Err(AUTDInternalError::SamplingFreqInvalid(512, ULTRASOUND_FREQ)), 512)]
    #[case::not_supported_max(
        Err(AUTDInternalError::SamplingFreqInvalid(
            sampling_config::freq_max() - 1,
            ULTRASOUND_FREQ
        )),
        sampling_config::freq_max() - 1
    )]
    #[case::out_of_range_max(
        Err(AUTDInternalError::SamplingFreqInvalid(
            sampling_config::freq_max() * 2,
            ULTRASOUND_FREQ
        )),
        sampling_config::freq_max() * 2
    )]
    fn from_freq(
        #[case] expected: Result<SamplingConfiguration, AUTDInternalError>,
        #[case] freq: u32,
    ) {
        assert_eq!(expected, SamplingConfiguration::from_freq(freq));
    }

    #[rstest::rstest]
    #[test]
    #[case::min(
        Ok(SamplingConfiguration::Frequency(Frequency(sampling_config::freq_min_raw()))),
        sampling_config::freq_min_raw()
    )]
    #[case::max(
        Ok(SamplingConfiguration::Frequency(Frequency(sampling_config::freq_max_raw()))),
        sampling_config::freq_max_raw()
    )]
    #[case(Ok(SamplingConfiguration::Frequency(Frequency(512.))), 512.)]
    #[case::not_supported_max(
        Ok(SamplingConfiguration::Frequency(Frequency(sampling_config::freq_max() as f64 - 1.))),
        sampling_config::freq_max() as f64 - 1.
    )]
    #[case::out_of_range_min(
        Err(AUTDInternalError::SamplingFreqOutOfRange(
            sampling_config::freq_min_raw() as f64 - f64::MIN,
            sampling_config::freq_min_raw(),
            sampling_config::freq_max_raw()
        )),
        sampling_config::freq_min_raw() as f64 - f64::MIN
    )]
    #[case::out_of_range_max(
        Err(AUTDInternalError::SamplingFreqOutOfRange(
            sampling_config::freq_max_raw() as f64 + f64::MIN,
            sampling_config::freq_min_raw(),
            sampling_config::freq_max_raw()
        )),
        sampling_config::freq_max_raw() as f64 + f64::MIN
    )]
    fn from_freq_nearest(
        #[case] expected: Result<SamplingConfiguration, AUTDInternalError>,
        #[case] freq: f64,
    ) {
        assert_eq!(expected, SamplingConfiguration::from_freq_nearest(freq));
    }

    #[rstest::rstest]
    #[test]
    #[case::min(
        Ok(SamplingConfiguration::Period(Period(sampling_config::period_min()))),
        sampling_config::period_min()
    )]
    #[case::max(
        Ok(SamplingConfiguration::Period(Period(sampling_config::period_max()))),
        sampling_config::period_max()
    )]
    #[case::not_supported_min(
        Err(AUTDInternalError::SamplingPeriodInvalid(
            sampling_config::period_min() + Duration::from_nanos(1),
            sampling_config::period_min()

        )),
        sampling_config::period_min() + Duration::from_nanos(1)
    )]
    #[case::not_supported_max(
        Err(AUTDInternalError::SamplingPeriodInvalid(
            sampling_config::period_max() - Duration::from_nanos(1),
            sampling_config::period_min()
        )),
        sampling_config::period_max() - Duration::from_nanos(1)
    )]
    #[case::out_of_range_min(
        Err(AUTDInternalError::SamplingPeriodInvalid(
            sampling_config::period_min() / 2,
            sampling_config::period_min()
        )),
        sampling_config::period_min() / 2
    )]
    #[case::out_of_range_max(
        Err(AUTDInternalError::SamplingPeriodOutOfRange(
            sampling_config::period_max() * 2,
            sampling_config::period_min_raw(),
            sampling_config::period_max_raw()
        )),
        sampling_config::period_max() * 2
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
        Ok(SamplingConfiguration::Period(Period(sampling_config::period_min_raw()))),
        sampling_config::period_min_raw()
    )]
    #[case::max(
        Ok(SamplingConfiguration::Period(Period(sampling_config::period_max_raw()))),
        sampling_config::period_max_raw()
    )]
    #[case::not_supported_min(
        Ok(SamplingConfiguration::Period(Period(sampling_config::period_min_raw() + Duration::from_nanos(1)))),
        sampling_config::period_min_raw() + Duration::from_nanos(1)
    )]
    #[case::not_supported_max(
        Ok(SamplingConfiguration::Period(Period(sampling_config::period_max_raw() - Duration::from_nanos(1)))),
        sampling_config::period_max_raw() - Duration::from_nanos(1)
    )]
    #[case::out_of_range_min(
        Err(AUTDInternalError::SamplingPeriodOutOfRange(
            sampling_config::period_min_raw() - Duration::from_nanos(1),
            sampling_config::period_min_raw(),
            sampling_config::period_max_raw()
        )),
        sampling_config::period_min_raw() - Duration::from_nanos(1)
    )]
    #[case::out_of_range_max(
        Err(AUTDInternalError::SamplingPeriodOutOfRange(
            sampling_config::period_max_raw() + Duration::from_nanos(1),
            sampling_config::period_min_raw(),
            sampling_config::period_max_raw()
        )),
        sampling_config::period_max_raw() + Duration::from_nanos(1)
    )]
    fn from_period_nearest(
        #[case] expected: Result<SamplingConfiguration, AUTDInternalError>,
        #[case] period: Duration,
    ) {
        assert_eq!(expected, SamplingConfiguration::from_period_nearest(period));
    }

    #[rstest::rstest]
    #[test]
    #[case::freq(4e3, SamplingConfiguration::Frequency(Frequency(4e3)))]
    #[case::period(4e3, SamplingConfiguration::Period(Period(Duration::from_micros(250))))]
    #[case::div(4e3, SamplingConfiguration::Division(Division(5120)))]
    fn freq(#[case] expect: f64, #[case] config: SamplingConfiguration) {
        assert_eq!(expect, config.freq());
    }

    #[rstest::rstest]
    #[test]
    #[case::freq(
        Duration::from_micros(250),
        SamplingConfiguration::Frequency(Frequency(4e3))
    )]
    #[case::period(
        Duration::from_micros(250),
        SamplingConfiguration::Period(Period(Duration::from_micros(250)))
    )]
    #[case::div(
        Duration::from_micros(250),
        SamplingConfiguration::Division(Division(5120))
    )]
    fn period(#[case] expect: Duration, #[case] config: SamplingConfiguration) {
        assert_eq!(expect, config.period());
    }

    #[rstest::rstest]
    #[test]
    #[case::freq(SamplingConfiguration::Frequency(Frequency(4e3)), "4000Hz")]
    #[case::period(
        SamplingConfiguration::Period(Period(Duration::from_micros(250))),
        "250µs"
    )]
    #[case::div(
        SamplingConfiguration::Division(Division(305419896)),
        "Division(305419896)"
    )]
    fn display(#[case] config: SamplingConfiguration, #[case] expected: &str) {
        assert_eq!(expected, config.to_string());
    }
}
