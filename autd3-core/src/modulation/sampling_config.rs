use std::{convert::Infallible, fmt::Debug, num::NonZeroU16};

use crate::{
    defined::{ultrasound_freq, Freq, Hz},
    utils::float::is_integer,
};

use super::error::SamplingConfigError;

/// The configuration for sampling.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct SamplingConfig {
    /// The division number of the sampling frequency.
    ///
    /// The sampling frequency is [`ultrasound_freq`] / `division`.
    pub division: NonZeroU16,
}

pub trait IntoSamplingConfig {
    type Error: std::error::Error;
    fn into_sampling_config(self) -> Result<SamplingConfig, Self::Error>;
}

impl IntoSamplingConfig for NonZeroU16 {
    type Error = Infallible;
    fn into_sampling_config(self) -> Result<SamplingConfig, Self::Error> {
        Ok(SamplingConfig { division: self })
    }
}

impl IntoSamplingConfig for u16 {
    type Error = SamplingConfigError;
    fn into_sampling_config(self) -> Result<SamplingConfig, Self::Error> {
        if self == 0 {
            return Err(SamplingConfigError::SamplingDivisionInvalid);
        }
        Ok(SamplingConfig {
            division: NonZeroU16::new(self).unwrap(),
        })
    }
}

impl IntoSamplingConfig for Freq<u32> {
    type Error = SamplingConfigError;
    fn into_sampling_config(self) -> Result<SamplingConfig, Self::Error> {
        const FREQ_MIN: Freq<u32> = Freq { freq: 1 };
        if !(FREQ_MIN..=ultrasound_freq()).contains(&self) {
            return Err(Self::Error::SamplingFreqOutOfRange(
                self,
                FREQ_MIN,
                ultrasound_freq(),
            ));
        }
        if ultrasound_freq().hz() % self.hz() != 0 {
            return Err(Self::Error::SamplingFreqInvalid(self));
        }
        Ok(SamplingConfig {
            division: NonZeroU16::new((ultrasound_freq().hz() / self.hz()) as _).unwrap(),
        })
    }
}

impl IntoSamplingConfig for Freq<f32> {
    type Error = SamplingConfigError;
    fn into_sampling_config(self) -> Result<SamplingConfig, Self::Error> {
        let freq_max = ultrasound_freq().hz() as f32 * Hz;
        let freq_min = freq_max / u16::MAX as f32;
        if !(freq_min..=freq_max).contains(&self) {
            return Err(Self::Error::SamplingFreqOutOfRangeF(
                self, freq_min, freq_max,
            ));
        }
        let division = ultrasound_freq().hz() as f32 / self.hz();
        if !is_integer(division as _) {
            return Err(Self::Error::SamplingFreqInvalidF(self));
        }
        Ok(SamplingConfig {
            division: NonZeroU16::new(division as _).unwrap(),
        })
    }
}

#[cfg(not(feature = "dynamic_freq"))]
impl IntoSamplingConfig for std::time::Duration {
    type Error = SamplingConfigError;
    fn into_sampling_config(self) -> Result<SamplingConfig, Self::Error> {
        use crate::defined::ultrasound_period;

        let period_min = ultrasound_period();
        let period_max = std::time::Duration::from_micros(
            u16::MAX as u64 * ultrasound_period().as_micros() as u64,
        );
        if !(period_min..=period_max).contains(&self) {
            return Err(Self::Error::SamplingPeriodOutOfRange(
                self, period_min, period_max,
            ));
        }
        if self.as_nanos() % ultrasound_period().as_nanos() != 0 {
            return Err(Self::Error::SamplingPeriodInvalid(self));
        }
        Ok(SamplingConfig {
            division: NonZeroU16::new((self.as_nanos() / ultrasound_period().as_nanos()) as _)
                .unwrap(),
        })
    }
}

pub trait IntoSamplingConfigNearest {
    fn into_sampling_config_nearest(self) -> SamplingConfig;
}

impl IntoSamplingConfigNearest for Freq<f32> {
    fn into_sampling_config_nearest(self) -> SamplingConfig {
        SamplingConfig {
            division: NonZeroU16::new(
                (ultrasound_freq().hz() as f32 / self.hz())
                    .clamp(1.0, u16::MAX as f32)
                    .round() as u16,
            )
            .unwrap(),
        }
    }
}

impl IntoSamplingConfigNearest for Freq<u32> {
    fn into_sampling_config_nearest(self) -> SamplingConfig {
        SamplingConfig {
            division: NonZeroU16::new(
                (ultrasound_freq().hz() + self.hz() / 2)
                    .checked_div(self.hz())
                    .unwrap_or(u32::MAX)
                    .clamp(1, u16::MAX as u32) as u16,
            )
            .unwrap(),
        }
    }
}

#[cfg(not(feature = "dynamic_freq"))]
impl IntoSamplingConfigNearest for std::time::Duration {
    fn into_sampling_config_nearest(self) -> SamplingConfig {
        use crate::defined::ultrasound_period;
        SamplingConfig {
            division: NonZeroU16::new(
                ((self.as_nanos() + ultrasound_period().as_nanos() / 2)
                    / ultrasound_period().as_nanos())
                .clamp(1, u16::MAX as u128) as u16,
            )
            .unwrap(),
        }
    }
}

impl SamplingConfig {
    /// A [`SamplingConfig`] of 40kHz.
    #[cfg(not(feature = "dynamic_freq"))]
    pub const FREQ_40K: SamplingConfig = SamplingConfig {
        division: NonZeroU16::MIN,
    };
    /// A [`SamplingConfig`] of 4kHz.
    #[cfg(not(feature = "dynamic_freq"))]
    pub const FREQ_4K: SamplingConfig = SamplingConfig {
        division: NonZeroU16::new(10).unwrap(),
    };
    /// A [`SamplingConfig`] of the minimum frequency.
    pub const FREQ_MIN: SamplingConfig = SamplingConfig {
        division: NonZeroU16::MAX,
    };
    /// A [`SamplingConfig`] of the maximum frequency, that is the ultrasound frequency.
    pub const FREQ_MAX: SamplingConfig = SamplingConfig {
        division: NonZeroU16::MIN,
    };
    /// A [`SamplingConfig`] of ultrasound frequency divided by 10.
    pub const DIV_10: SamplingConfig = SamplingConfig {
        division: NonZeroU16::new(10).unwrap(),
    };

    /// Creates a new [`SamplingConfig`].
    pub fn new<T: IntoSamplingConfig>(value: T) -> Result<Self, T::Error> {
        value.into_sampling_config()
    }

    /// Creates a new [`SamplingConfig`] with the nearest frequency or period value of the possible values.
    pub fn new_nearest(value: impl IntoSamplingConfigNearest) -> Self {
        value.into_sampling_config_nearest()
    }

    /// Gets the sampling frequency.
    pub fn freq(&self) -> Freq<f32> {
        ultrasound_freq().hz() as f32 / self.division.get() as f32 * Hz
    }

    /// Gets the sampling period.
    #[cfg(not(feature = "dynamic_freq"))]
    pub fn period(&self) -> std::time::Duration {
        crate::defined::ultrasound_period() * self.division.get() as u32
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

impl TryInto<SamplingConfig> for Result<SamplingConfig, SamplingConfigError> {
    type Error = SamplingConfigError;

    fn try_into(self) -> Result<SamplingConfig, Self::Error> {
        self
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
    #[case(Ok(1), NonZeroU16::MIN)]
    #[case(Ok(u16::MAX), NonZeroU16::MAX)]
    #[case(Ok(1), 40000 * Hz)]
    #[case(Ok(10), 4000 * Hz)]
    #[case(Ok(1), 40000. * Hz)]
    #[case(Ok(10), 4000. * Hz)]
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
    fn division<E: Debug + PartialEq>(
        #[case] expect: Result<u16, E>,
        #[case] value: impl IntoSamplingConfig<Error = E>,
    ) {
        assert_eq!(expect, SamplingConfig::new(value).map(|c| c.division.get()));
    }

    #[rstest::rstest]
    #[test]
    #[case(Ok(40000. * Hz), NonZeroU16::MIN)]
    #[case(Ok(0.61036086 * Hz), NonZeroU16::MAX)]
    #[case(Ok(40000. * Hz), 40000 * Hz)]
    #[case(Ok(4000. * Hz), 4000 * Hz)]
    #[case(Ok(40000. * Hz), 40000. * Hz)]
    #[case(Ok(4000. * Hz), 4000. * Hz)]
    #[cfg_attr(not(feature = "dynamic_freq"), case(Ok(40000. * Hz), Duration::from_micros(25)))]
    #[cfg_attr(not(feature = "dynamic_freq"), case(Ok(4000. * Hz), Duration::from_micros(250)))]
    fn freq<E: Debug + PartialEq>(
        #[case] expect: Result<Freq<f32>, E>,
        #[case] value: impl IntoSamplingConfig<Error = E>,
    ) {
        assert_eq!(expect, SamplingConfig::new(value).map(|c| c.freq()));
    }

    #[cfg(not(feature = "dynamic_freq"))]
    #[rstest::rstest]
    #[test]
    #[case(Ok(Duration::from_micros(25)), NonZeroU16::MIN)]
    #[case(Ok(Duration::from_micros(1638375)), NonZeroU16::MAX)]
    #[case(Ok(Duration::from_micros(25)), 40000 * Hz)]
    #[case(Ok(Duration::from_micros(250)), 4000 * Hz)]
    #[case(Ok(Duration::from_micros(25)), 40000. * Hz)]
    #[case(Ok(Duration::from_micros(250)), 4000. * Hz)]
    #[case(Ok(Duration::from_micros(25)), Duration::from_micros(25))]
    #[case(Ok(Duration::from_micros(250)), Duration::from_micros(250))]
    fn period<E: Debug + PartialEq>(
        #[case] expect: Result<Duration, E>,
        #[case] value: impl IntoSamplingConfig<Error = E>,
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
        assert_eq!(expected, SamplingConfig::new_nearest(freq).division.get());
    }

    #[rstest::rstest]
    #[test]
    #[case::min(40000, 1 * Hz)]
    #[case::max(1, 40000 * Hz)]
    #[case::not_supported_max(1, ultrasound_freq() - 1 * Hz)]
    #[case::out_of_range_min(0xFFFF, 0 * Hz)]
    #[case::out_of_range_max(1, ultrasound_freq() + 1 * Hz)]
    fn from_freq_u32_nearest(#[case] expected: u16, #[case] freq: Freq<u32>) {
        assert_eq!(expected, SamplingConfig::new_nearest(freq).division.get());
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
        assert_eq!(expected, SamplingConfig::new_nearest(p).division.get());
    }
}
