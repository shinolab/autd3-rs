use std::fmt::Debug;

use autd3_derive::Builder;

use crate::{
    defined::{ultrasound_freq, Freq, Hz},
    utils::float::is_integer,
};

use super::error::SamplingConfigError;

/// The configuration for sampling.
#[derive(Clone, Copy, Debug, PartialEq, Builder)]
#[repr(C)]
pub struct SamplingConfig {
    #[get]
    /// The division number of the sampling frequency.
    ///
    /// The sampling frequency is [`ultrasound_freq`] / `division`.
    division: u16,
}

pub trait IntoSamplingConfig {
    fn into_sampling_config(self) -> Result<SamplingConfig, SamplingConfigError>;
}

impl IntoSamplingConfig for u16 {
    fn into_sampling_config(self) -> Result<SamplingConfig, SamplingConfigError> {
        if self == 0 {
            return Err(SamplingConfigError::SamplingDivisionInvalid(self));
        }
        Ok(SamplingConfig { division: self })
    }
}

impl IntoSamplingConfig for Freq<u32> {
    fn into_sampling_config(self) -> Result<SamplingConfig, SamplingConfigError> {
        const FREQ_MIN: Freq<u32> = Freq { freq: 1 };
        if !(FREQ_MIN..=ultrasound_freq()).contains(&self) {
            return Err(SamplingConfigError::SamplingFreqOutOfRange(
                self,
                FREQ_MIN,
                ultrasound_freq(),
            ));
        }
        if ultrasound_freq().hz() % self.hz() != 0 {
            return Err(SamplingConfigError::SamplingFreqInvalid(self));
        }
        Ok(SamplingConfig {
            division: (ultrasound_freq().hz() / self.hz()) as _,
        })
    }
}

impl IntoSamplingConfig for Freq<f32> {
    fn into_sampling_config(self) -> Result<SamplingConfig, SamplingConfigError> {
        let freq_max = ultrasound_freq().hz() as f32 * Hz;
        let freq_min = freq_max / u16::MAX as f32;
        if !(freq_min..=freq_max).contains(&self) {
            return Err(SamplingConfigError::SamplingFreqOutOfRangeF(
                self, freq_min, freq_max,
            ));
        }
        let div = ultrasound_freq().hz() as f32 / self.hz();
        if !is_integer(div as _) {
            return Err(SamplingConfigError::SamplingFreqInvalidF(self));
        }
        Ok(SamplingConfig { division: div as _ })
    }
}

#[cfg(not(feature = "dynamic_freq"))]
impl IntoSamplingConfig for std::time::Duration {
    fn into_sampling_config(self) -> Result<SamplingConfig, SamplingConfigError> {
        use crate::defined::ultrasound_period;

        let period_min = ultrasound_period();
        let period_max = std::time::Duration::from_micros(
            u16::MAX as u64 * ultrasound_period().as_micros() as u64,
        );
        if !(period_min..=period_max).contains(&self) {
            return Err(SamplingConfigError::SamplingPeriodOutOfRange(
                self, period_min, period_max,
            ));
        }
        if self.as_nanos() % ultrasound_period().as_nanos() != 0 {
            return Err(SamplingConfigError::SamplingPeriodInvalid(self));
        }
        Ok(SamplingConfig {
            division: (self.as_nanos() / ultrasound_period().as_nanos()) as _,
        })
    }
}

pub trait IntoSamplingConfigNearest {
    fn into_sampling_config_nearest(self) -> SamplingConfig;
}

impl IntoSamplingConfigNearest for Freq<f32> {
    fn into_sampling_config_nearest(self) -> SamplingConfig {
        SamplingConfig::new(
            (ultrasound_freq().hz() as f32 / self.hz())
                .clamp(1.0, u16::MAX as f32)
                .round() as u16,
        )
        .unwrap()
    }
}

impl IntoSamplingConfigNearest for Freq<u32> {
    fn into_sampling_config_nearest(self) -> SamplingConfig {
        SamplingConfig::new(
            (ultrasound_freq().hz() + self.hz() / 2)
                .checked_div(self.hz())
                .unwrap_or(u32::MAX)
                .clamp(1, u16::MAX as u32) as u16,
        )
        .unwrap()
    }
}

#[cfg(not(feature = "dynamic_freq"))]
impl IntoSamplingConfigNearest for std::time::Duration {
    fn into_sampling_config_nearest(self) -> SamplingConfig {
        use crate::defined::ultrasound_period;

        SamplingConfig::new(
            ((self.as_nanos() + ultrasound_period().as_nanos() / 2)
                / ultrasound_period().as_nanos())
            .clamp(1, u16::MAX as u128) as u16,
        )
        .unwrap()
    }
}

impl SamplingConfig {
    /// A [`SamplingConfig`] of 40kHz.
    pub const FREQ_40K: SamplingConfig = SamplingConfig { division: 1 };
    /// A [`SamplingConfig`] of 4kHz.
    pub const FREQ_4K: SamplingConfig = SamplingConfig { division: 10 };
    /// A [`SamplingConfig`] of the minimum frequency.
    pub const FREQ_MIN: SamplingConfig = SamplingConfig { division: u16::MAX };

    /// Creates a new [`SamplingConfig`].
    pub fn new(value: impl IntoSamplingConfig) -> Result<Self, SamplingConfigError> {
        value.into_sampling_config()
    }

    /// Creates a new [`SamplingConfig`] with the nearest frequency or period value of the possible values.
    pub fn new_nearest(value: impl IntoSamplingConfigNearest) -> Self {
        value.into_sampling_config_nearest()
    }

    /// Gets the sampling frequency.
    pub fn freq(&self) -> Freq<f32> {
        ultrasound_freq().hz() as f32 / self.division() as f32 * Hz
    }

    /// Gets the sampling period.
    #[cfg(not(feature = "dynamic_freq"))]
    pub fn period(&self) -> std::time::Duration {
        crate::defined::ultrasound_period() * self.division() as u32
    }
}

// GRCOV_EXCL_START
impl TryInto<SamplingConfig> for Freq<u32> {
    type Error = SamplingConfigError;

    fn try_into(self) -> Result<SamplingConfig, Self::Error> {
        SamplingConfig::new(self)
    }
}

impl TryInto<SamplingConfig> for Freq<f32> {
    type Error = SamplingConfigError;

    fn try_into(self) -> Result<SamplingConfig, Self::Error> {
        SamplingConfig::new(self)
    }
}

#[cfg(not(feature = "dynamic_freq"))]
impl TryInto<SamplingConfig> for std::time::Duration {
    type Error = SamplingConfigError; 

    fn try_into(self) -> Result<SamplingConfig, Self::Error> {
        SamplingConfig::new(self)
    }
}
// GRCOV_EXCL_STOP

#[cfg(test)]
mod tests {
    use crate::defined::Hz;

    #[cfg(not(feature = "dynamic_freq"))]
    use crate::defined::ultrasound_period;
    #[cfg(not(feature = "dynamic_freq"))]
    use std::time::Duration;

    use super::*;

    #[rstest::rstest]
    #[test]
    #[case(Ok(1), 1)]
    #[case(Ok(u16::MAX), u16::MAX)]
    #[case(Ok(1), 40000 * Hz)]
    #[case(Ok(10), 4000 * Hz)]
    #[case(Ok(1), 40000. * Hz)]
    #[case(Ok(10), 4000. * Hz)]
    #[case(Err(SamplingConfigError::SamplingDivisionInvalid(0)), 0)]
    #[case(Err(SamplingConfigError::SamplingFreqInvalid(ultrasound_freq() - 1 * Hz)), ultrasound_freq() - 1 * Hz)]
    #[case(Err(SamplingConfigError::SamplingFreqOutOfRange(0 * Hz, 1 * Hz, ultrasound_freq())), 0 * Hz)]
    #[case(Err(SamplingConfigError::SamplingFreqOutOfRange(ultrasound_freq() + 1 * Hz, 1 * Hz, ultrasound_freq())), ultrasound_freq() + 1 * Hz)]
    #[case(Err(SamplingConfigError::SamplingFreqInvalidF((ultrasound_freq().hz() as f32 - 1.) * Hz)), (ultrasound_freq().hz() as f32 - 1.) * Hz)]
    #[case(Err(SamplingConfigError::SamplingFreqOutOfRangeF(0. * Hz, ultrasound_freq().hz() as f32 * Hz / u16::MAX as f32, ultrasound_freq().hz() as f32 * Hz)), 0. * Hz)]
    #[case(Err(SamplingConfigError::SamplingFreqOutOfRangeF(40000. * Hz + 1. * Hz, ultrasound_freq().hz() as f32 * Hz / u16::MAX as f32, ultrasound_freq().hz() as f32 * Hz)), 40000. * Hz + 1. * Hz)]
    #[cfg_attr(not(feature = "dynamic_freq"), case(Ok(1), Duration::from_micros(25)))]
    #[cfg_attr(
        not(feature = "dynamic_freq"),
        case(Ok(10), Duration::from_micros(250))
    )]
    #[cfg_attr(not(feature = "dynamic_freq"), case(Err(SamplingConfigError::SamplingPeriodInvalid(Duration::from_micros(u16::MAX as u64 * ultrasound_period().as_micros() as u64) - Duration::from_nanos(1))), Duration::from_micros(u16::MAX as u64 * ultrasound_period().as_micros() as u64) - Duration::from_nanos(1)))]
    #[cfg_attr(not(feature = "dynamic_freq"), case(Err(SamplingConfigError::SamplingPeriodOutOfRange(ultrasound_period() / 2, ultrasound_period(), Duration::from_micros(u16::MAX as u64 * ultrasound_period().as_micros() as u64))), ultrasound_period() / 2))]
    #[cfg_attr(not(feature = "dynamic_freq"), case(Err(SamplingConfigError::SamplingPeriodOutOfRange(Duration::from_micros(u16::MAX as u64 * ultrasound_period().as_micros() as u64) * 2, ultrasound_period(), Duration::from_micros(u16::MAX as u64 * ultrasound_period().as_micros() as u64))), Duration::from_micros(u16::MAX as u64 * ultrasound_period().as_micros() as u64) * 2))]
    fn division(
        #[case] expect: Result<u16, SamplingConfigError>,
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
    #[cfg_attr(not(feature = "dynamic_freq"), case(Ok(40000. * Hz), Duration::from_micros(25)))]
    #[cfg_attr(not(feature = "dynamic_freq"), case(Ok(4000. * Hz), Duration::from_micros(250)))]
    fn freq(
        #[case] expect: Result<Freq<f32>, SamplingConfigError>,
        #[case] value: impl IntoSamplingConfig,
    ) {
        assert_eq!(expect, SamplingConfig::new(value).map(|c| c.freq()));
    }

    #[cfg(not(feature = "dynamic_freq"))]
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
        #[case] expect: Result<Duration, SamplingConfigError>,
        #[case] value: impl IntoSamplingConfig,
    ) {
        assert_eq!(expect, SamplingConfig::new(value).map(|c| c.period()));
    }

    #[rstest::rstest]
    #[test]
    #[case::min(u16::MAX, (40000. / u16::MAX as f32) * Hz)]
    #[case::max(1, 40000. * Hz)]
    #[case::not_supported_max(1, (ultrasound_freq().hz() as f32 - 1.) * Hz)]
    #[case::out_of_range_min(u16::MAX, 0. * Hz)]
    #[case::out_of_range_max(1, 40000. * Hz + 1. * Hz)]
    fn from_freq_f32_nearest(#[case] expected: u16, #[case] freq: Freq<f32>) {
        assert_eq!(expected, SamplingConfig::new_nearest(freq).division());
    }

    #[rstest::rstest]
    #[test]
    #[case::min(40000, 1 * Hz)]
    #[case::max(1, 40000 * Hz)]
    #[case::not_supported_max(1, ultrasound_freq() - 1 * Hz)]
    #[case::out_of_range_min(0xFFFF, 0 * Hz)]
    #[case::out_of_range_max(1, ultrasound_freq() + 1 * Hz)]
    fn from_freq_u32_nearest(#[case] expected: u16, #[case] freq: Freq<u32>) {
        assert_eq!(expected, SamplingConfig::new_nearest(freq).division());
    }

    #[cfg(not(feature = "dynamic_freq"))]
    #[rstest::rstest]
    #[test]
    #[case::min(1, ultrasound_period())]
    #[case::max(u16::MAX, Duration::from_micros(u16::MAX as u64 * ultrasound_period().as_micros() as u64))]
    #[case::not_supported_max(u16::MAX, Duration::from_micros(u16::MAX as u64 * ultrasound_period().as_micros() as u64) - Duration::from_nanos(1))]
    #[case::out_of_range_min(1, ultrasound_period() / 2)]
    #[case::out_of_range_max(u16::MAX, Duration::from_micros(u16::MAX as u64 * ultrasound_period().as_micros() as u64) * 2)]
    fn from_period_nearest(#[case] expected: u16, #[case] p: Duration) {
        assert_eq!(expected, SamplingConfig::new_nearest(p).division());
    }
}
