use std::{fmt::Debug, num::NonZeroU16};

use crate::{
    defined::{ultrasound_freq, Freq, Hz},
    utils::float::is_integer,
};

use super::error::SamplingConfigError;

/// Nearest type.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Nearest<T: Copy + Clone + Debug + PartialEq>(pub T);

/// The configuration for sampling.
#[derive(Clone, Copy, PartialEq)]
pub enum SamplingConfig {
    #[doc(hidden)]
    Division(NonZeroU16),
    #[doc(hidden)]
    Freq(Freq<f32>),
    #[cfg(not(feature = "dynamic_freq"))]
    #[doc(hidden)]
    Period(std::time::Duration),
    #[doc(hidden)]
    FreqNearest(Nearest<Freq<f32>>),
    #[cfg(not(feature = "dynamic_freq"))]
    #[doc(hidden)]
    PeriodNearest(Nearest<std::time::Duration>),
}

impl std::fmt::Debug for SamplingConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SamplingConfig::Division(div) => write!(f, "SamplingConfig::Division({})", div),
            SamplingConfig::Freq(freq) => write!(f, "SamplingConfig::Freq({:?})", freq),
            #[cfg(not(feature = "dynamic_freq"))]
            SamplingConfig::Period(period) => write!(f, "SamplingConfig::Period({:?})", period),
            SamplingConfig::FreqNearest(nearest) => {
                write!(f, "SamplingConfig::FreqNearest({:?})", nearest)
            }
            #[cfg(not(feature = "dynamic_freq"))]
            SamplingConfig::PeriodNearest(nearest) => {
                write!(f, "SamplingConfig::PeriodNearest({:?})", nearest)
            }
        }
    }
}

impl From<NonZeroU16> for SamplingConfig {
    fn from(value: NonZeroU16) -> Self {
        Self::Division(value)
    }
}

impl From<Freq<f32>> for SamplingConfig {
    fn from(value: Freq<f32>) -> Self {
        Self::Freq(value)
    }
}

#[cfg(not(feature = "dynamic_freq"))]
impl From<std::time::Duration> for SamplingConfig {
    fn from(value: std::time::Duration) -> Self {
        Self::Period(value)
    }
}

impl SamplingConfig {
    /// A [`SamplingConfig`] of 40kHz.
    pub const FREQ_40K: Self = SamplingConfig::Freq(Freq { freq: 40000. });
    /// A [`SamplingConfig`] of 4kHz.
    pub const FREQ_4K: Self = SamplingConfig::Freq(Freq { freq: 4000. });

    /// Creates a new [`SamplingConfig`].
    pub fn new(value: impl Into<SamplingConfig>) -> Self {
        value.into()
    }

    /// The division number of the sampling frequency.
    ///
    /// The sampling frequency is [`ultrasound_freq`] / `division`.
    pub fn division(&self) -> Result<u16, SamplingConfigError> {
        match *self {
            SamplingConfig::Division(div) => Ok(div.get()),
            SamplingConfig::Freq(freq) => {
                let freq_max = ultrasound_freq().hz() as f32 * Hz;
                let freq_min = freq_max / u16::MAX as f32;
                if !(freq_min..=freq_max).contains(&freq) {
                    return Err(SamplingConfigError::FreqOutOfRangeF(
                        freq, freq_min, freq_max,
                    ));
                }
                let division = ultrasound_freq().hz() as f32 / freq.hz();
                if !is_integer(division as _) {
                    return Err(SamplingConfigError::FreqInvalidF(freq));
                }
                Ok(division as _)
            }
            #[cfg(not(feature = "dynamic_freq"))]
            SamplingConfig::Period(duration) => {
                use crate::defined::ultrasound_period;

                let period_min = ultrasound_period();
                let period_max = std::time::Duration::from_micros(
                    u16::MAX as u64 * ultrasound_period().as_micros() as u64,
                );
                if !(period_min..=period_max).contains(&duration) {
                    return Err(SamplingConfigError::PeriodOutOfRange(
                        duration, period_min, period_max,
                    ));
                }
                if duration.as_nanos() % ultrasound_period().as_nanos() != 0 {
                    return Err(SamplingConfigError::PeriodInvalid(duration));
                }
                Ok((duration.as_nanos() / ultrasound_period().as_nanos()) as _)
            }
            SamplingConfig::FreqNearest(nearest) => Ok((ultrasound_freq().hz() as f32
                / nearest.0.hz())
            .clamp(1.0, u16::MAX as f32)
            .round() as u16),
            #[cfg(not(feature = "dynamic_freq"))]
            SamplingConfig::PeriodNearest(nearest) => {
                use crate::defined::ultrasound_period;

                Ok(((nearest.0.as_nanos() + ultrasound_period().as_nanos() / 2)
                    / ultrasound_period().as_nanos())
                .clamp(1, u16::MAX as u128) as u16)
            }
        }
    }

    /// The sampling frequency.
    pub fn freq(&self) -> Result<Freq<f32>, SamplingConfigError> {
        Ok(ultrasound_freq().hz() as f32 / self.division()? as f32 * Hz)
    }

    #[cfg(not(feature = "dynamic_freq"))]
    /// The sampling period.
    pub fn period(&self) -> Result<std::time::Duration, SamplingConfigError> {
        Ok(crate::defined::ultrasound_period() * self.division()? as u32)
    }
}

impl SamplingConfig {
    /// Converts to a [`SamplingConfig`] with the nearest frequency or period among the possible values.
    pub fn into_nearest(self) -> SamplingConfig {
        match self {
            SamplingConfig::Freq(freq) => SamplingConfig::FreqNearest(Nearest(freq)),
            #[cfg(not(feature = "dynamic_freq"))]
            SamplingConfig::Period(period) => SamplingConfig::PeriodNearest(Nearest(period)),
            _ => self,
        }
    }
}

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
    #[case(Ok(1), 40000. * Hz)]
    #[case(Ok(10), 4000. * Hz)]
    #[case(Err(SamplingConfigError::FreqInvalidF((ultrasound_freq().hz() as f32 - 1.) * Hz)), (ultrasound_freq().hz() as f32 - 1.) * Hz)]
    #[case(Err(SamplingConfigError::FreqOutOfRangeF(0. * Hz, ultrasound_freq().hz() as f32 * Hz / u16::MAX as f32, ultrasound_freq().hz() as f32 * Hz)), 0. * Hz)]
    #[case(Err(SamplingConfigError::FreqOutOfRangeF(40000. * Hz + 1. * Hz, ultrasound_freq().hz() as f32 * Hz / u16::MAX as f32, ultrasound_freq().hz() as f32 * Hz)), 40000. * Hz + 1. * Hz)]
    #[cfg_attr(not(feature = "dynamic_freq"), case(Ok(1), Duration::from_micros(25)))]
    #[cfg_attr(
        not(feature = "dynamic_freq"),
        case(Ok(10), Duration::from_micros(250))
    )]
    #[cfg_attr(not(feature = "dynamic_freq"), case(Err(SamplingConfigError::PeriodInvalid(Duration::from_micros(u16::MAX as u64 * ultrasound_period().as_micros() as u64) - Duration::from_nanos(1))), Duration::from_micros(u16::MAX as u64 * ultrasound_period().as_micros() as u64) - Duration::from_nanos(1)))]
    #[cfg_attr(not(feature = "dynamic_freq"), case(Err(SamplingConfigError::PeriodOutOfRange(ultrasound_period() / 2, ultrasound_period(), Duration::from_micros(u16::MAX as u64 * ultrasound_period().as_micros() as u64))), ultrasound_period() / 2))]
    #[cfg_attr(not(feature = "dynamic_freq"), case(Err(SamplingConfigError::PeriodOutOfRange(Duration::from_micros(u16::MAX as u64 * ultrasound_period().as_micros() as u64) * 2, ultrasound_period(), Duration::from_micros(u16::MAX as u64 * ultrasound_period().as_micros() as u64))), Duration::from_micros(u16::MAX as u64 * ultrasound_period().as_micros() as u64) * 2))]
    fn division(
        #[case] expect: Result<u16, SamplingConfigError>,
        #[case] value: impl Into<SamplingConfig>,
    ) {
        assert_eq!(expect, SamplingConfig::new(value).division());
    }

    #[rstest::rstest]
    #[test]
    #[case(Ok(40000. * Hz), NonZeroU16::MIN)]
    #[case(Ok(0.61036086 * Hz), NonZeroU16::MAX)]
    #[case(Ok(40000. * Hz), 40000. * Hz)]
    #[case(Ok(4000. * Hz), 4000. * Hz)]
    #[cfg_attr(not(feature = "dynamic_freq"), case(Ok(40000. * Hz), Duration::from_micros(25)))]
    #[cfg_attr(not(feature = "dynamic_freq"), case(Ok(4000. * Hz), Duration::from_micros(250)))]
    fn freq(
        #[case] expect: Result<Freq<f32>, SamplingConfigError>,
        #[case] value: impl Into<SamplingConfig>,
    ) {
        assert_eq!(expect, SamplingConfig::new(value).freq());
    }

    #[cfg(not(feature = "dynamic_freq"))]
    #[rstest::rstest]
    #[test]
    #[case(Ok(Duration::from_micros(25)), NonZeroU16::MIN)]
    #[case(Ok(Duration::from_micros(1638375)), NonZeroU16::MAX)]
    #[case(Ok(Duration::from_micros(25)), 40000. * Hz)]
    #[case(Ok(Duration::from_micros(250)), 4000. * Hz)]
    #[case(Ok(Duration::from_micros(25)), Duration::from_micros(25))]
    #[case(Ok(Duration::from_micros(250)), Duration::from_micros(250))]
    fn period(
        #[case] expect: Result<Duration, SamplingConfigError>,
        #[case] value: impl Into<SamplingConfig>,
    ) {
        assert_eq!(expect, SamplingConfig::new(value).period());
    }

    #[rstest::rstest]
    #[test]
    #[case::min(u16::MAX, (40000. / u16::MAX as f32) * Hz)]
    #[case::max(1, 40000. * Hz)]
    #[case::not_supported_max(1, (ultrasound_freq().hz() as f32 - 1.) * Hz)]
    #[case::out_of_range_min(u16::MAX, 0. * Hz)]
    #[case::out_of_range_max(1, 40000. * Hz + 1. * Hz)]
    fn from_freq_nearest(#[case] expected: u16, #[case] freq: Freq<f32>) {
        assert_eq!(
            Ok(expected),
            SamplingConfig::new(freq).into_nearest().division()
        );
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
        assert_eq!(
            Ok(expected),
            SamplingConfig::new(p).into_nearest().division()
        );
    }
}
