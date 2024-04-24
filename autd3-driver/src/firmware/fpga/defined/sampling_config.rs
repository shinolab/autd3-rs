use crate::{
    error::AUTDInternalError,
    firmware::fpga::{FPGA_CLK_FREQ, SAMPLING_FREQ_DIV_MAX, SAMPLING_FREQ_DIV_MIN},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SamplingConfiguration {
    div: u32,
}

impl SamplingConfiguration {
    pub const BASE_FREQUENCY: f64 = FPGA_CLK_FREQ as _;

    pub const FREQ_MIN: f64 = Self::BASE_FREQUENCY / SAMPLING_FREQ_DIV_MAX as f64;
    pub const FREQ_MAX: f64 = Self::BASE_FREQUENCY / SAMPLING_FREQ_DIV_MIN as f64;
    pub const PERIOD_MIN: u128 =
        (1000000000. / Self::BASE_FREQUENCY * SAMPLING_FREQ_DIV_MIN as f64) as u128;
    pub const PERIOD_MAX: u128 = 209715199999;

    pub const DISABLE: Self = Self { div: 0xFFFFFFFF };
    pub const FREQ_4K_HZ: Self = Self { div: 5120 };

    pub fn from_frequency_division(div: u32) -> Result<Self, AUTDInternalError> {
        if !(SAMPLING_FREQ_DIV_MIN..=SAMPLING_FREQ_DIV_MAX).contains(&div) {
            Err(AUTDInternalError::SamplingFreqDivOutOfRange(
                div,
                SAMPLING_FREQ_DIV_MIN,
                SAMPLING_FREQ_DIV_MAX,
            ))
        } else {
            Ok(Self { div })
        }
    }

    pub fn from_frequency(f: f64) -> Result<Self, AUTDInternalError> {
        Self::from_frequency_division((Self::BASE_FREQUENCY / f) as _).map_err(|_| {
            AUTDInternalError::SamplingFreqOutOfRange(f as _, Self::FREQ_MIN, Self::FREQ_MAX)
        })
    }

    pub fn from_period(p: std::time::Duration) -> Result<Self, AUTDInternalError> {
        let p = p.as_nanos();
        let div = (Self::BASE_FREQUENCY * (p as f64 / 1000000000.)) as u64;
        if div > SAMPLING_FREQ_DIV_MAX as u64 {
            return Err(AUTDInternalError::SamplingPeriodOutOfRange(
                p,
                Self::PERIOD_MIN,
                Self::PERIOD_MAX,
            ));
        }
        Self::from_frequency_division(div as _).map_err(|_| {
            AUTDInternalError::SamplingPeriodOutOfRange(p, Self::PERIOD_MIN, Self::PERIOD_MAX)
        })
    }

    pub const fn frequency_division(&self) -> u32 {
        self.div
    }

    pub fn frequency(&self) -> f64 {
        (Self::BASE_FREQUENCY / self.div as f64) as _
    }

    pub fn period(&self) -> std::time::Duration {
        let p = 1000000000. / Self::BASE_FREQUENCY * self.div as f64;
        std::time::Duration::from_nanos(p as _)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[test]
    #[case::min(
        Ok(SamplingConfiguration {
            div: SAMPLING_FREQ_DIV_MIN
        }),
        SAMPLING_FREQ_DIV_MIN
    )]
    #[case::max(
        Ok(SamplingConfiguration {
            div: SAMPLING_FREQ_DIV_MAX
        }),
        SAMPLING_FREQ_DIV_MAX
    )]
    #[case::out_of_range(
        Err(AUTDInternalError::SamplingFreqDivOutOfRange(
            SAMPLING_FREQ_DIV_MIN - 1,
            SAMPLING_FREQ_DIV_MIN,
            SAMPLING_FREQ_DIV_MAX
        )),
        SAMPLING_FREQ_DIV_MIN - 1
    )]
    fn test_from_frequency_division(
        #[case] expected: Result<SamplingConfiguration, AUTDInternalError>,
        #[case] freq_div: u32,
    ) {
        assert_eq!(
            expected,
            SamplingConfiguration::from_frequency_division(freq_div)
        );
    }

    #[rstest::rstest]
    #[test]
    #[case::min(
        Ok(SamplingConfiguration {
            div: SAMPLING_FREQ_DIV_MIN
        }),
        SamplingConfiguration::FREQ_MAX
    )]
    #[case::max(
        Ok(SamplingConfiguration {
            div: SAMPLING_FREQ_DIV_MAX
        }),
        SamplingConfiguration::FREQ_MIN
    )]
    #[case::out_of_range_min(
        Err(AUTDInternalError::SamplingFreqOutOfRange(
            SamplingConfiguration::FREQ_MIN - 0.1,
            SamplingConfiguration::FREQ_MIN,
            SamplingConfiguration::FREQ_MAX
        )),
        SamplingConfiguration::FREQ_MIN - 0.1
    )]
    #[case::out_of_range_max(
        Err(AUTDInternalError::SamplingFreqOutOfRange(
            SamplingConfiguration::FREQ_MAX + 0.1,
            SamplingConfiguration::FREQ_MIN,
            SamplingConfiguration::FREQ_MAX
        )),
        SamplingConfiguration::FREQ_MAX + 0.1
    )]
    fn test_from_frequency(
        #[case] expected: Result<SamplingConfiguration, AUTDInternalError>,
        #[case] freq: f64,
    ) {
        assert_eq!(expected, SamplingConfiguration::from_frequency(freq));
    }

    #[rstest::rstest]
    #[test]
    #[case::min(
        Ok(SamplingConfiguration {
            div: SAMPLING_FREQ_DIV_MIN
        }),
        SamplingConfiguration::PERIOD_MIN
    )]
    #[case::max(
        Ok(SamplingConfiguration {
            div: SAMPLING_FREQ_DIV_MAX
        }),
        SamplingConfiguration::PERIOD_MAX
    )]
    #[case::out_of_range_min(
        Err(AUTDInternalError::SamplingPeriodOutOfRange(
            SamplingConfiguration::PERIOD_MIN - 1,
            SamplingConfiguration::PERIOD_MIN,
            SamplingConfiguration::PERIOD_MAX
        )),
        SamplingConfiguration::PERIOD_MIN - 1
    )]
    #[case::out_of_range_max(
        Err(AUTDInternalError::SamplingPeriodOutOfRange(
            SamplingConfiguration::PERIOD_MAX + 1,
            SamplingConfiguration::PERIOD_MIN,
            SamplingConfiguration::PERIOD_MAX
        )),
        SamplingConfiguration::PERIOD_MAX + 1
    )]
    fn test_from_period(
        #[case] expected: Result<SamplingConfiguration, AUTDInternalError>,
        #[case] period: u128,
    ) {
        assert_eq!(
            expected,
            SamplingConfiguration::from_period(std::time::Duration::from_nanos(period as _))
        );
    }
}
