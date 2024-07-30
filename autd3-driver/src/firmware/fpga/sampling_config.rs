use std::{fmt::Debug, num::NonZeroU16, time::Duration};

use crate::{
    defined::{Freq, Hz, ULTRASOUND_FREQ},
    error::AUTDInternalError,
    utils::float::is_integer,
};

const NANOSEC: u128 = 1_000_000_000;

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct SamplingConfig {
    div: NonZeroU16,
}

const FREQ_MIN: Freq<u32> = Freq { freq: 1 };
const FREQ_MAX: Freq<u32> = ULTRASOUND_FREQ;
const FREQ_MIN_F: Freq<f32> = Freq {
    freq: 40000. / u16::MAX as f32,
};
const FREQ_MAX_F: Freq<f32> = Freq { freq: 40000. };
const PERIOD_MIN: Duration = Duration::from_nanos((NANOSEC / ULTRASOUND_FREQ.hz() as u128) as u64);
const PERIOD_MAX: Duration =
    Duration::from_nanos((u16::MAX as u128 * NANOSEC / ULTRASOUND_FREQ.hz() as u128) as u64);

impl SamplingConfig {
    pub const FREQ_40K: SamplingConfig = SamplingConfig {
        div: unsafe { NonZeroU16::new_unchecked(1) },
    };
    pub const FREQ_4K: SamplingConfig = SamplingConfig {
        div: unsafe { NonZeroU16::new_unchecked(10) },
    };

    pub const fn new(div: NonZeroU16) -> Self {
        Self { div }
    }

    const unsafe fn new_unchecked(div: u16) -> Self {
        Self {
            div: NonZeroU16::new_unchecked(div),
        }
    }

    pub const fn division(self) -> u16 {
        self.div.get()
    }

    pub fn freq(&self) -> Freq<f32> {
        ULTRASOUND_FREQ.hz() as f32 / self.division() as f32 * Hz
    }

    pub const fn period(&self) -> Duration {
        Duration::from_nanos(
            (self.division() as u128 * NANOSEC / ULTRASOUND_FREQ.hz() as u128) as u64,
        )
    }
}

impl std::fmt::Display for SamplingConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.freq())
    }
}

pub trait IntoSamplingConfigNearest {
    fn into_sampling_config_nearest(self) -> SamplingConfig;
}

impl IntoSamplingConfigNearest for Freq<f32> {
    fn into_sampling_config_nearest(self) -> SamplingConfig {
        unsafe {
            SamplingConfig::new_unchecked(
                (ULTRASOUND_FREQ.hz() as f32 / self.hz())
                    .clamp(1.0, u16::MAX as f32)
                    .round() as u16,
            )
        }
    }
}

impl IntoSamplingConfigNearest for Freq<u32> {
    fn into_sampling_config_nearest(self) -> SamplingConfig {
        unsafe {
            SamplingConfig::new_unchecked(
                (ULTRASOUND_FREQ.hz() + self.hz() / 2)
                    .checked_div(self.hz())
                    .unwrap_or(u32::MAX)
                    .clamp(1, u16::MAX as u32) as u16,
            )
        }
    }
}

impl IntoSamplingConfigNearest for Duration {
    fn into_sampling_config_nearest(self) -> SamplingConfig {
        unsafe {
            SamplingConfig::new_unchecked(
                ((self.as_nanos() * ULTRASOUND_FREQ.hz() as u128 + NANOSEC / 2) / NANOSEC)
                    .clamp(1, u16::MAX as u128) as u16,
            )
        }
    }
}

pub trait IntoSamplingConfig {
    fn into_sampling_config(self) -> Result<SamplingConfig, AUTDInternalError>;
}

impl IntoSamplingConfig for SamplingConfig {
    fn into_sampling_config(self) -> Result<SamplingConfig, AUTDInternalError> {
        Ok(self)
    }
}

impl IntoSamplingConfig for Freq<u32> {
    fn into_sampling_config(self) -> Result<SamplingConfig, AUTDInternalError> {
        if !(FREQ_MIN..=FREQ_MAX).contains(&self) {
            return Err(AUTDInternalError::SamplingFreqOutOfRange(
                self, FREQ_MIN, FREQ_MAX,
            ));
        }
        if ULTRASOUND_FREQ.hz() % self.hz() != 0 {
            return Err(AUTDInternalError::SamplingFreqInvalid(self));
        }
        Ok(unsafe { SamplingConfig::new_unchecked((ULTRASOUND_FREQ.hz() / self.hz()) as u16) })
    }
}

impl IntoSamplingConfig for Freq<f32> {
    fn into_sampling_config(self) -> Result<SamplingConfig, AUTDInternalError> {
        if !(FREQ_MIN_F..=FREQ_MAX_F).contains(&self) {
            return Err(AUTDInternalError::SamplingFreqOutOfRangeF(
                self, FREQ_MIN_F, FREQ_MAX_F,
            ));
        }
        let div = ULTRASOUND_FREQ.hz() as f32 / self.hz();
        if !is_integer(div as _) {
            return Err(AUTDInternalError::SamplingFreqInvalidF(self));
        }
        Ok(unsafe { SamplingConfig::new_unchecked(div as u16) })
    }
}

impl IntoSamplingConfig for Duration {
    fn into_sampling_config(self) -> Result<SamplingConfig, AUTDInternalError> {
        if !(PERIOD_MIN..=PERIOD_MAX).contains(&self) {
            return Err(AUTDInternalError::SamplingPeriodOutOfRange(
                self, PERIOD_MIN, PERIOD_MAX,
            ));
        }
        let div = self.as_nanos() * ULTRASOUND_FREQ.hz() as u128;
        if div % NANOSEC != 0 {
            return Err(AUTDInternalError::SamplingPeriodInvalid(self));
        }

        Ok(unsafe { SamplingConfig::new_unchecked((div / NANOSEC) as _) })
    }
}

#[cfg(test)]
mod tests {
    use crate::defined::{kHz, Hz};

    use super::*;

    #[rstest::rstest]
    #[test]
    #[case::min(NonZeroU16::MIN)]
    #[case::max(NonZeroU16::MAX)]
    #[cfg_attr(miri, ignore)]
    fn new(#[case] div: NonZeroU16) {
        assert_eq!(div, SamplingConfig::new(div).div);
    }

    #[rstest::rstest]
    #[test]
    #[case(1, SamplingConfig::new(NonZeroU16::MIN))]
    #[case(u16::MAX, SamplingConfig::new(NonZeroU16::MAX))]
    #[case(1, SamplingConfig::FREQ_40K)]
    #[case(10, SamplingConfig::FREQ_4K)]
    #[cfg_attr(miri, ignore)]
    fn division(#[case] expected: u16, #[case] config: SamplingConfig) {
        assert_eq!(expected, config.division());
    }

    #[rstest::rstest]
    #[test]
    #[case(40. * kHz, SamplingConfig::new(NonZeroU16::MIN))]
    #[case(0.61036086 * Hz, SamplingConfig::new(NonZeroU16::MAX))]
    #[case(40. * kHz, SamplingConfig::FREQ_40K)]
    #[case(4. * kHz, SamplingConfig::FREQ_4K)]
    #[cfg_attr(miri, ignore)]
    fn freq(#[case] expected: Freq<f32>, #[case] config: SamplingConfig) {
        assert_eq!(expected, config.freq());
    }

    #[rstest::rstest]
    #[test]
    #[case(Duration::from_micros(25), SamplingConfig::new(NonZeroU16::MIN))]
    #[case(Duration::from_micros(1638375), SamplingConfig::new(NonZeroU16::MAX))]
    #[case(Duration::from_micros(25), SamplingConfig::FREQ_40K)]
    #[case(Duration::from_micros(250), SamplingConfig::FREQ_4K)]
    #[cfg_attr(miri, ignore)]
    fn period(#[case] expected: Duration, #[case] config: SamplingConfig) {
        assert_eq!(expected, config.period());
    }

    #[rstest::rstest]
    #[test]
    #[case::min(u16::MAX, (40000. / u16::MAX as f32) * Hz)]
    #[case::max(1, 40000. * Hz)]
    #[case::not_supported_max(1, (ULTRASOUND_FREQ.hz() as f32 - 1.) * Hz)]
    #[case::out_of_range_min(u16::MAX, 0. * Hz)]
    #[case::out_of_range_max(1, 40000. * Hz + 1. * Hz)]
    #[cfg_attr(miri, ignore)]
    fn from_freq_f32_nearest(#[case] expected: u16, #[case] freq: Freq<f32>) {
        assert_eq!(expected, freq.into_sampling_config_nearest().division());
    }

    #[rstest::rstest]
    #[test]
    #[case::min(40000, 1 * Hz)]
    #[case::max(1, 40000 * Hz)]
    #[case::not_supported_max(1, ULTRASOUND_FREQ - 1 * Hz)]
    #[case::out_of_range_min(0xFFFF, 0 * Hz)]
    #[case::out_of_range_max(1, ULTRASOUND_FREQ + 1 * Hz)]
    #[cfg_attr(miri, ignore)]
    fn from_freq_u32_nearest(#[case] expected: u16, #[case] freq: Freq<u32>) {
        assert_eq!(expected, freq.into_sampling_config_nearest().division());
    }

    #[rstest::rstest]
    #[test]
    #[case::min(Ok(u16::MAX), (40000. / u16::MAX as f32) * Hz)]
    #[case::max(Ok(1), 40000. * Hz)]
    #[case::not_supported_max(Err(AUTDInternalError::SamplingFreqInvalidF((ULTRASOUND_FREQ.hz() as f32 - 1.) * Hz)), (ULTRASOUND_FREQ.hz() as f32 - 1.) * Hz)]
    #[case::out_of_range_min(Err(AUTDInternalError::SamplingFreqOutOfRangeF(0. * Hz, FREQ_MIN_F, FREQ_MAX_F)), 0. * Hz)]
    #[case::out_of_range_max(Err(AUTDInternalError::SamplingFreqOutOfRangeF(40000. * Hz + 1. * Hz, FREQ_MIN_F, FREQ_MAX_F)), 40000. * Hz + 1. * Hz)]
    #[cfg_attr(miri, ignore)]
    fn try_from_freq_f32(
        #[case] expected: Result<u16, AUTDInternalError>,
        #[case] freq: Freq<f32>,
    ) {
        assert_eq!(expected, freq.into_sampling_config().map(|c| c.division()));
    }

    #[rstest::rstest]
    #[test]
    #[case::min(Ok(40000), 1 * Hz)]
    #[case::max(Ok(1), 40000 * Hz)]
    #[case::not_supported_max(Err(AUTDInternalError::SamplingFreqInvalid(ULTRASOUND_FREQ - 1 * Hz)), ULTRASOUND_FREQ - 1 * Hz)]
    #[case::out_of_range_min(Err(AUTDInternalError::SamplingFreqOutOfRange(0 * Hz, FREQ_MIN, FREQ_MAX)), 0 * Hz)]
    #[case::out_of_range_max(Err(AUTDInternalError::SamplingFreqOutOfRange(ULTRASOUND_FREQ + 1 * Hz, FREQ_MIN, FREQ_MAX)), ULTRASOUND_FREQ + 1 * Hz)]
    #[cfg_attr(miri, ignore)]
    fn try_from_freq_u32(
        #[case] expected: Result<u16, AUTDInternalError>,
        #[case] freq: Freq<u32>,
    ) {
        assert_eq!(expected, freq.into_sampling_config().map(|c| c.division()));
    }

    #[rstest::rstest]
    #[test]
    #[case::min(1, PERIOD_MIN)]
    #[case::max(u16::MAX, PERIOD_MAX)]
    #[case::not_supported_max(u16::MAX, PERIOD_MAX - Duration::from_nanos(1))]
    #[case::out_of_range_min(1, PERIOD_MIN / 2)]
    #[case::out_of_range_max(u16::MAX, PERIOD_MAX * 2)]
    #[cfg_attr(miri, ignore)]
    fn from_period_nearest(#[case] expected: u16, #[case] p: Duration) {
        assert_eq!(expected, p.into_sampling_config_nearest().division());
    }

    #[rstest::rstest]
    #[test]
    #[case::min(Ok(1), PERIOD_MIN)]
    #[case::max(Ok(u16::MAX), PERIOD_MAX)]
    #[case::not_supported_max(Err(AUTDInternalError::SamplingPeriodInvalid(PERIOD_MAX - Duration::from_nanos(1))), PERIOD_MAX - Duration::from_nanos(1))]
    #[case::out_of_range_min(Err(AUTDInternalError::SamplingPeriodOutOfRange(PERIOD_MIN / 2, PERIOD_MIN, PERIOD_MAX)), PERIOD_MIN / 2)]
    #[case::out_of_range_max(Err(AUTDInternalError::SamplingPeriodOutOfRange(PERIOD_MAX * 2, PERIOD_MIN, PERIOD_MAX)), PERIOD_MAX * 2)]
    #[cfg_attr(miri, ignore)]
    fn try_from_period(#[case] expected: Result<u16, AUTDInternalError>, #[case] p: Duration) {
        assert_eq!(expected, p.into_sampling_config().map(|c| c.division()));
    }

    #[rstest::rstest]
    #[test]
    #[case::freq("40000 Hz", SamplingConfig::FREQ_40K)]
    #[case::freq("4000 Hz", SamplingConfig::FREQ_4K)]
    fn display(#[case] expected: &str, #[case] config: SamplingConfig) {
        assert_eq!(expected, config.to_string());
    }
}
