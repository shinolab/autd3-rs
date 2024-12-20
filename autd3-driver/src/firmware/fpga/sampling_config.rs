use std::{fmt::Debug, time::Duration};

use autd3_derive::Builder;

use crate::{
    defined::{Freq, Hz, ULTRASOUND_FREQ, ULTRASOUND_PERIOD},
    error::AUTDDriverError,
    utils::float::is_integer,
};

#[derive(Clone, Copy, Debug, PartialEq, Builder)]
#[repr(C)]
pub struct SamplingConfig {
    #[get]
    division: u16,
}

pub trait IntoSamplingConfig {
    fn into_sampling_config(self) -> Result<SamplingConfig, AUTDDriverError>;
}

impl IntoSamplingConfig for u16 {
    fn into_sampling_config(self) -> Result<SamplingConfig, AUTDDriverError> {
        if self == 0 {
            return Err(AUTDDriverError::SamplingDivisionInvalid(self));
        }
        Ok(SamplingConfig { division: self })
    }
}

impl IntoSamplingConfig for Freq<u32> {
    fn into_sampling_config(self) -> Result<SamplingConfig, AUTDDriverError> {
        if !(FREQ_MIN..=FREQ_MAX).contains(&self) {
            return Err(AUTDDriverError::SamplingFreqOutOfRange(
                self, FREQ_MIN, FREQ_MAX,
            ));
        }
        if ULTRASOUND_FREQ.hz() % self.hz() != 0 {
            return Err(AUTDDriverError::SamplingFreqInvalid(self));
        }
        Ok(SamplingConfig {
            division: (ULTRASOUND_FREQ.hz() / self.hz()) as _,
        })
    }
}

impl IntoSamplingConfig for Freq<f32> {
    fn into_sampling_config(self) -> Result<SamplingConfig, AUTDDriverError> {
        if !(FREQ_MIN_F..=FREQ_MAX_F).contains(&self) {
            return Err(AUTDDriverError::SamplingFreqOutOfRangeF(
                self, FREQ_MIN_F, FREQ_MAX_F,
            ));
        }
        let div = ULTRASOUND_FREQ.hz() as f32 / self.hz();
        if !is_integer(div as _) {
            return Err(AUTDDriverError::SamplingFreqInvalidF(self));
        }
        Ok(SamplingConfig { division: div as _ })
    }
}

impl IntoSamplingConfig for Duration {
    fn into_sampling_config(self) -> Result<SamplingConfig, AUTDDriverError> {
        if !(PERIOD_MIN..=PERIOD_MAX).contains(&self) {
            return Err(AUTDDriverError::SamplingPeriodOutOfRange(
                self, PERIOD_MIN, PERIOD_MAX,
            ));
        }
        if self.as_nanos() % ULTRASOUND_PERIOD.as_nanos() != 0 {
            return Err(AUTDDriverError::SamplingPeriodInvalid(self));
        }
        Ok(SamplingConfig {
            division: (self.as_nanos() / ULTRASOUND_PERIOD.as_nanos()) as _,
        })
    }
}

pub trait IntoSamplingConfigNearest {
    fn into_sampling_config_nearest(self) -> SamplingConfig;
}

impl IntoSamplingConfigNearest for Freq<f32> {
    fn into_sampling_config_nearest(self) -> SamplingConfig {
        SamplingConfig::new(
            (ULTRASOUND_FREQ.hz() as f32 / self.hz())
                .clamp(1.0, u16::MAX as f32)
                .round() as u16,
        )
        .unwrap()
    }
}

impl IntoSamplingConfigNearest for Freq<u32> {
    fn into_sampling_config_nearest(self) -> SamplingConfig {
        SamplingConfig::new(
            (ULTRASOUND_FREQ.hz() + self.hz() / 2)
                .checked_div(self.hz())
                .unwrap_or(u32::MAX)
                .clamp(1, u16::MAX as u32) as u16,
        )
        .unwrap()
    }
}

impl IntoSamplingConfigNearest for Duration {
    fn into_sampling_config_nearest(self) -> SamplingConfig {
        SamplingConfig::new(
            ((self.as_nanos() + ULTRASOUND_PERIOD.as_nanos() / 2) / ULTRASOUND_PERIOD.as_nanos())
                .clamp(1, u16::MAX as u128) as u16,
        )
        .unwrap()
    }
}

const FREQ_MIN: Freq<u32> = Freq { freq: 1 };
const FREQ_MAX: Freq<u32> = ULTRASOUND_FREQ;
const FREQ_MIN_F: Freq<f32> = Freq {
    freq: 40000. / u16::MAX as f32,
};
const FREQ_MAX_F: Freq<f32> = Freq { freq: 40000. };
const PERIOD_MIN: Duration = ULTRASOUND_PERIOD;
const PERIOD_MAX: Duration =
    Duration::from_micros(u16::MAX as u64 * ULTRASOUND_PERIOD.as_micros() as u64);

impl SamplingConfig {
    pub const FREQ_40K: SamplingConfig = SamplingConfig { division: 1 };
    pub const FREQ_4K: SamplingConfig = SamplingConfig { division: 10 };
    pub const FREQ_MIN: SamplingConfig = SamplingConfig { division: u16::MAX };

    pub fn new(value: impl IntoSamplingConfig) -> Result<Self, AUTDDriverError> {
        value.into_sampling_config()
    }

    pub fn new_nearest(value: impl IntoSamplingConfigNearest) -> Self {
        value.into_sampling_config_nearest()
    }

    pub fn freq(&self) -> Freq<f32> {
        ULTRASOUND_FREQ.hz() as f32 / self.division() as f32 * Hz
    }

    pub fn period(&self) -> Duration {
        ULTRASOUND_PERIOD * self.division() as u32
    }
}

// GRCOV_EXCL_START
impl TryInto<SamplingConfig> for Freq<u32> {
    type Error = AUTDDriverError;

    fn try_into(self) -> Result<SamplingConfig, Self::Error> {
        SamplingConfig::new(self)
    }
}

impl TryInto<SamplingConfig> for Freq<f32> {
    type Error = AUTDDriverError;

    fn try_into(self) -> Result<SamplingConfig, Self::Error> {
        SamplingConfig::new(self)
    }
}

impl TryInto<SamplingConfig> for Duration {
    type Error = AUTDDriverError;

    fn try_into(self) -> Result<SamplingConfig, Self::Error> {
        SamplingConfig::new(self)
    }
}
// GRCOV_EXCL_STOP

#[cfg(test)]
mod tests {
    use crate::defined::Hz;

    use super::*;

    #[rstest::rstest]
    #[test]
    #[case(Ok(1), 1)]
    #[case(Ok(u16::MAX), u16::MAX)]
    #[case(Ok(1), 40000 * Hz)]
    #[case(Ok(10), 4000 * Hz)]
    #[case(Ok(1), 40000. * Hz)]
    #[case(Ok(10), 4000. * Hz)]
    #[case(Ok(1), Duration::from_micros(25))]
    #[case(Ok(10), Duration::from_micros(250))]
    #[case(Err(AUTDDriverError::SamplingDivisionInvalid(0)), 0)]
    #[case(Err(AUTDDriverError::SamplingFreqInvalid(ULTRASOUND_FREQ - 1 * Hz)), ULTRASOUND_FREQ - 1 * Hz)]
    #[case(Err(AUTDDriverError::SamplingFreqOutOfRange(0 * Hz, FREQ_MIN, FREQ_MAX)), 0 * Hz)]
    #[case(Err(AUTDDriverError::SamplingFreqOutOfRange(ULTRASOUND_FREQ + 1 * Hz, FREQ_MIN, FREQ_MAX)), ULTRASOUND_FREQ + 1 * Hz)]
    #[case(Err(AUTDDriverError::SamplingFreqInvalidF((ULTRASOUND_FREQ.hz() as f32 - 1.) * Hz)), (ULTRASOUND_FREQ.hz() as f32 - 1.) * Hz)]
    #[case(Err(AUTDDriverError::SamplingFreqOutOfRangeF(0. * Hz, FREQ_MIN_F, FREQ_MAX_F)), 0. * Hz)]
    #[case(Err(AUTDDriverError::SamplingFreqOutOfRangeF(40000. * Hz + 1. * Hz, FREQ_MIN_F, FREQ_MAX_F)), 40000. * Hz + 1. * Hz)]
    #[case(Err(AUTDDriverError::SamplingPeriodInvalid(PERIOD_MAX - Duration::from_nanos(1))), PERIOD_MAX - Duration::from_nanos(1))]
    #[case(Err(AUTDDriverError::SamplingPeriodOutOfRange(PERIOD_MIN / 2, PERIOD_MIN, PERIOD_MAX)), PERIOD_MIN / 2)]
    #[case(Err(AUTDDriverError::SamplingPeriodOutOfRange(PERIOD_MAX * 2, PERIOD_MIN, PERIOD_MAX)), PERIOD_MAX * 2)]
    fn division(
        #[case] expect: Result<u16, AUTDDriverError>,
        #[case] value: impl IntoSamplingConfig,
    ) {
        assert_eq!(expect, SamplingConfig::new(value).map(|c| c.division()));
    }

    #[rstest::rstest]
    #[test]
    #[case(Ok(40000. * Hz), 1)]
    #[case(Ok(0.61036086 * Hz), u16::MAX)]
    #[case(Ok(40000. * Hz), 40000 * Hz)]
    #[case(Ok(4000. * Hz), 4000 * Hz)]
    #[case(Ok(40000. * Hz), 40000. * Hz)]
    #[case(Ok(4000. * Hz), 4000. * Hz)]
    #[case(Ok(40000. * Hz), Duration::from_micros(25))]
    #[case(Ok(4000. * Hz), Duration::from_micros(250))]
    fn freq(
        #[case] expect: Result<Freq<f32>, AUTDDriverError>,
        #[case] value: impl IntoSamplingConfig,
    ) {
        assert_eq!(expect, SamplingConfig::new(value).map(|c| c.freq()));
    }

    #[rstest::rstest]
    #[test]
    #[case(Ok(Duration::from_micros(25)), 1)]
    #[case(Ok(Duration::from_micros(1638375)), u16::MAX)]
    #[case(Ok(Duration::from_micros(25)), 40000 * Hz)]
    #[case(Ok(Duration::from_micros(250)), 4000 * Hz)]
    #[case(Ok(Duration::from_micros(25)), 40000. * Hz)]
    #[case(Ok(Duration::from_micros(250)), 4000. * Hz)]
    #[case(Ok(Duration::from_micros(25)), Duration::from_micros(25))]
    #[case(Ok(Duration::from_micros(250)), Duration::from_micros(250))]
    fn period(
        #[case] expect: Result<Duration, AUTDDriverError>,
        #[case] value: impl IntoSamplingConfig,
    ) {
        assert_eq!(expect, SamplingConfig::new(value).map(|c| c.period()));
    }

    #[rstest::rstest]
    #[test]
    #[case::min(u16::MAX, (40000. / u16::MAX as f32) * Hz)]
    #[case::max(1, 40000. * Hz)]
    #[case::not_supported_max(1, (ULTRASOUND_FREQ.hz() as f32 - 1.) * Hz)]
    #[case::out_of_range_min(u16::MAX, 0. * Hz)]
    #[case::out_of_range_max(1, 40000. * Hz + 1. * Hz)]
    fn from_freq_f32_nearest(#[case] expected: u16, #[case] freq: Freq<f32>) {
        assert_eq!(expected, SamplingConfig::new_nearest(freq).division());
    }

    #[rstest::rstest]
    #[test]
    #[case::min(40000, 1 * Hz)]
    #[case::max(1, 40000 * Hz)]
    #[case::not_supported_max(1, ULTRASOUND_FREQ - 1 * Hz)]
    #[case::out_of_range_min(0xFFFF, 0 * Hz)]
    #[case::out_of_range_max(1, ULTRASOUND_FREQ + 1 * Hz)]
    fn from_freq_u32_nearest(#[case] expected: u16, #[case] freq: Freq<u32>) {
        assert_eq!(expected, SamplingConfig::new_nearest(freq).division());
    }

    #[rstest::rstest]
    #[test]
    #[case::min(1, PERIOD_MIN)]
    #[case::max(u16::MAX, PERIOD_MAX)]
    #[case::not_supported_max(u16::MAX, PERIOD_MAX - Duration::from_nanos(1))]
    #[case::out_of_range_min(1, PERIOD_MIN / 2)]
    #[case::out_of_range_max(u16::MAX, PERIOD_MAX * 2)]
    fn from_period_nearest(#[case] expected: u16, #[case] p: Duration) {
        assert_eq!(expected, SamplingConfig::new_nearest(p).division());
    }
}
