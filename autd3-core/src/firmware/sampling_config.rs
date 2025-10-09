use core::{convert::Infallible, fmt::Debug, num::NonZeroU16, time::Duration};

use crate::{
    common::{Freq, Hz, ULTRASOUND_FREQ},
    utils::float::is_integer,
};

#[derive(Debug, PartialEq, Copy, Clone)]
/// An error produced by the sampling configuration.
pub enum SamplingConfigError {
    /// Invalid sampling divide.
    DivideInvalid,
    /// Invalid sampling frequency.
    FreqInvalid(Freq<u32>),
    /// Invalid sampling frequency.
    FreqInvalidF(Freq<f32>),
    /// Invalid sampling period.
    PeriodInvalid(Duration),
    /// Sampling frequency is out of range.
    FreqOutOfRange(Freq<u32>, Freq<u32>, Freq<u32>),
    /// Sampling frequency is out of range.
    FreqOutOfRangeF(Freq<f32>, Freq<f32>, Freq<f32>),
    /// Sampling period is out of range.
    PeriodOutOfRange(Duration, Duration, Duration),
}

// GRCOV_EXCL_START
impl core::fmt::Display for SamplingConfigError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SamplingConfigError::DivideInvalid => write!(f, "Sampling divide must not be zero"),
            SamplingConfigError::FreqInvalid(freq) => {
                write!(
                    f,
                    "Sampling frequency ({:?}) must divide the ultrasound frequency",
                    freq
                )
            }
            SamplingConfigError::FreqInvalidF(freq) => {
                write!(
                    f,
                    "Sampling frequency ({:?}) must divide the ultrasound frequency",
                    freq
                )
            }
            SamplingConfigError::PeriodInvalid(period) => {
                write!(
                    f,
                    "Sampling period ({:?}) must be a multiple of the ultrasound period",
                    period
                )
            }
            SamplingConfigError::FreqOutOfRange(freq, min, max) => {
                write!(
                    f,
                    "Sampling frequency ({:?}) is out of range ([{:?}, {:?}])",
                    freq, min, max
                )
            }
            SamplingConfigError::FreqOutOfRangeF(freq, min, max) => {
                write!(
                    f,
                    "Sampling frequency ({:?}) is out of range ([{:?}, {:?}])",
                    freq, min, max
                )
            }
            SamplingConfigError::PeriodOutOfRange(period, min, max) => {
                write!(
                    f,
                    "Sampling period ({:?}) is out of range ([{:?}, {:?}])",
                    period, min, max
                )
            }
        }
    }
}

impl core::error::Error for SamplingConfigError {}

impl From<Infallible> for SamplingConfigError {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}
// GRCOV_EXCL_STOP

/// Nearest type.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Nearest<T: Copy + Clone + Debug + PartialEq>(pub T);

/// The configuration for sampling.
#[derive(Clone, Copy)]
pub enum SamplingConfig {
    #[doc(hidden)]
    Divide(NonZeroU16),
    #[doc(hidden)]
    Freq(Freq<f32>),
    #[doc(hidden)]
    Period(core::time::Duration),
    #[doc(hidden)]
    FreqNearest(Nearest<Freq<f32>>),
    #[doc(hidden)]
    PeriodNearest(Nearest<core::time::Duration>),
}

impl PartialEq for SamplingConfig {
    fn eq(&self, other: &Self) -> bool {
        match (self.divide(), other.divide()) {
            (Ok(lhs), Ok(rhs)) => lhs == rhs,
            _ => false,
        }
    }
}

impl core::fmt::Debug for SamplingConfig {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SamplingConfig::Divide(div) => write!(f, "SamplingConfig::Divide({div})"),
            SamplingConfig::Freq(freq) => write!(f, "SamplingConfig::Freq({freq:?})"),
            SamplingConfig::Period(period) => write!(f, "SamplingConfig::Period({period:?})"),
            SamplingConfig::FreqNearest(nearest) => {
                write!(f, "SamplingConfig::FreqNearest({nearest:?})")
            }
            SamplingConfig::PeriodNearest(nearest) => {
                write!(f, "SamplingConfig::PeriodNearest({nearest:?})")
            }
        }
    }
}

impl From<NonZeroU16> for SamplingConfig {
    fn from(value: NonZeroU16) -> Self {
        Self::Divide(value)
    }
}

impl From<Freq<f32>> for SamplingConfig {
    fn from(value: Freq<f32>) -> Self {
        Self::Freq(value)
    }
}

impl From<core::time::Duration> for SamplingConfig {
    fn from(value: core::time::Duration) -> Self {
        Self::Period(value)
    }
}

impl SamplingConfig {
    /// A [`SamplingConfig`] of 40kHz.
    pub const FREQ_40K: Self = SamplingConfig::Freq(Freq { freq: 40000. });
    /// A [`SamplingConfig`] of 4kHz.
    pub const FREQ_4K: Self = SamplingConfig::Freq(Freq { freq: 4000. });

    /// Creates a new [`SamplingConfig`].
    #[must_use]
    pub fn new(value: impl Into<SamplingConfig>) -> Self {
        value.into()
    }

    /// The divide number of the sampling frequency.
    ///
    /// The sampling frequency is [`ULTRASOUND_FREQ`] / `divide`.
    pub fn divide(&self) -> Result<u16, SamplingConfigError> {
        match *self {
            SamplingConfig::Divide(div) => Ok(div.get()),
            SamplingConfig::Freq(freq) => {
                let freq_max = ULTRASOUND_FREQ.hz() as f32 * Hz;
                let freq_min = freq_max / u16::MAX as f32;
                if !(freq_min..=freq_max).contains(&freq) {
                    return Err(SamplingConfigError::FreqOutOfRangeF(
                        freq, freq_min, freq_max,
                    ));
                }
                let divide = ULTRASOUND_FREQ.hz() as f32 / freq.hz();
                if !is_integer(divide as _) {
                    return Err(SamplingConfigError::FreqInvalidF(freq));
                }
                Ok(divide as _)
            }
            SamplingConfig::Period(duration) => {
                use crate::common::ULTRASOUND_PERIOD;

                let period_min = ULTRASOUND_PERIOD;
                let period_max = core::time::Duration::from_micros(
                    u16::MAX as u64 * ULTRASOUND_PERIOD.as_micros() as u64,
                );
                if !(period_min..=period_max).contains(&duration) {
                    return Err(SamplingConfigError::PeriodOutOfRange(
                        duration, period_min, period_max,
                    ));
                }
                if duration.as_nanos() % ULTRASOUND_PERIOD.as_nanos() != 0 {
                    return Err(SamplingConfigError::PeriodInvalid(duration));
                }
                Ok((duration.as_nanos() / ULTRASOUND_PERIOD.as_nanos()) as _)
            }
            SamplingConfig::FreqNearest(nearest) => Ok(((ULTRASOUND_FREQ.hz() as f32
                / nearest.0.hz())
            .clamp(1.0, u16::MAX as f32))
            .round() as u16),
            SamplingConfig::PeriodNearest(nearest) => {
                use crate::common::ULTRASOUND_PERIOD;

                Ok(((nearest.0.as_nanos() + ULTRASOUND_PERIOD.as_nanos() / 2)
                    / ULTRASOUND_PERIOD.as_nanos())
                .clamp(1, u16::MAX as u128) as u16)
            }
        }
    }

    /// The sampling frequency.
    pub fn freq(&self) -> Result<Freq<f32>, SamplingConfigError> {
        Ok(ULTRASOUND_FREQ.hz() as f32 / self.divide()? as f32 * Hz)
    }

    /// The sampling period.
    pub fn period(&self) -> Result<core::time::Duration, SamplingConfigError> {
        Ok(crate::common::ULTRASOUND_PERIOD * self.divide()? as u32)
    }
}

impl SamplingConfig {
    /// Converts to a [`SamplingConfig`] with the nearest frequency or period among the possible values.
    #[must_use]
    pub const fn into_nearest(self) -> SamplingConfig {
        match self {
            SamplingConfig::Freq(freq) => SamplingConfig::FreqNearest(Nearest(freq)),
            SamplingConfig::Period(period) => SamplingConfig::PeriodNearest(Nearest(period)),
            _ => self,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::common::{Hz, kHz};

    use crate::common::ULTRASOUND_PERIOD;
    use core::time::Duration;

    use super::*;

    #[rstest::rstest]
    #[case(Ok(1), NonZeroU16::MIN)]
    #[case(Ok(u16::MAX), NonZeroU16::MAX)]
    #[case(Ok(1), 40000. * Hz)]
    #[case(Ok(10), 4000. * Hz)]
    #[case(Err(SamplingConfigError::FreqInvalidF((ULTRASOUND_FREQ.hz() as f32 - 1.) * Hz)), (ULTRASOUND_FREQ.hz() as f32 - 1.) * Hz)]
    #[case(Err(SamplingConfigError::FreqOutOfRangeF(0. * Hz, ULTRASOUND_FREQ.hz() as f32 * Hz / u16::MAX as f32, ULTRASOUND_FREQ.hz() as f32 * Hz)), 0. * Hz)]
    #[case(Err(SamplingConfigError::FreqOutOfRangeF(40000. * Hz + 1. * Hz, ULTRASOUND_FREQ.hz() as f32 * Hz / u16::MAX as f32, ULTRASOUND_FREQ.hz() as f32 * Hz)), 40000. * Hz + 1. * Hz)]
    #[case(Ok(1), Duration::from_micros(25))]
    #[case(Ok(10), Duration::from_micros(250))]
    #[case(Err(SamplingConfigError::PeriodInvalid(Duration::from_micros(u16::MAX as u64 * ULTRASOUND_PERIOD.as_micros() as u64) - Duration::from_nanos(1))), Duration::from_micros(u16::MAX as u64 * ULTRASOUND_PERIOD.as_micros() as u64) - Duration::from_nanos(1))]
    #[case(Err(SamplingConfigError::PeriodOutOfRange(ULTRASOUND_PERIOD / 2, ULTRASOUND_PERIOD, Duration::from_micros(u16::MAX as u64 * ULTRASOUND_PERIOD.as_micros() as u64))), ULTRASOUND_PERIOD / 2)]
    #[case(Err(SamplingConfigError::PeriodOutOfRange(Duration::from_micros(u16::MAX as u64 * ULTRASOUND_PERIOD.as_micros() as u64) * 2, ULTRASOUND_PERIOD, Duration::from_micros(u16::MAX as u64 * ULTRASOUND_PERIOD.as_micros() as u64))), Duration::from_micros(u16::MAX as u64 * ULTRASOUND_PERIOD.as_micros() as u64) * 2)]
    fn divide(
        #[case] expect: Result<u16, SamplingConfigError>,
        #[case] value: impl Into<SamplingConfig>,
    ) {
        assert_eq!(expect, SamplingConfig::new(value).divide());
    }

    #[rstest::rstest]
    #[case(Ok(40000. * Hz), NonZeroU16::MIN)]
    #[case(Ok(0.61036086 * Hz), NonZeroU16::MAX)]
    #[case(Ok(40000. * Hz), 40000. * Hz)]
    #[case(Ok(4000. * Hz), 4000. * Hz)]
    #[case(Ok(40000. * Hz), Duration::from_micros(25))]
    #[case(Ok(4000. * Hz), Duration::from_micros(250))]
    fn freq(
        #[case] expect: Result<Freq<f32>, SamplingConfigError>,
        #[case] value: impl Into<SamplingConfig>,
    ) {
        assert_eq!(expect, SamplingConfig::new(value).freq());
    }

    #[rstest::rstest]
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
    #[case::min(u16::MAX, (40000. / u16::MAX as f32) * Hz)]
    #[case::max(1, 40000. * Hz)]
    #[case::not_supported_max(1, (ULTRASOUND_FREQ.hz() as f32 - 1.) * Hz)]
    #[case::out_of_range_min(u16::MAX, 0. * Hz)]
    #[case::out_of_range_max(1, 40000. * Hz + 1. * Hz)]
    fn from_freq_nearest(#[case] expected: u16, #[case] freq: Freq<f32>) {
        assert_eq!(
            Ok(expected),
            SamplingConfig::new(freq).into_nearest().divide()
        );
    }

    #[rstest::rstest]
    #[case::min(1, ULTRASOUND_PERIOD)]
    #[case::max(u16::MAX, Duration::from_micros(u16::MAX as u64 * ULTRASOUND_PERIOD.as_micros() as u64))]
    #[case::not_supported_max(u16::MAX, Duration::from_micros(u16::MAX as u64 * ULTRASOUND_PERIOD.as_micros() as u64) - Duration::from_nanos(1))]
    #[case::out_of_range_min(1, ULTRASOUND_PERIOD / 2)]
    #[case::out_of_range_max(u16::MAX, Duration::from_micros(u16::MAX as u64 * ULTRASOUND_PERIOD.as_micros() as u64) * 2)]
    fn from_period_nearest(#[case] expected: u16, #[case] p: Duration) {
        assert_eq!(Ok(expected), SamplingConfig::new(p).into_nearest().divide());
    }

    #[rstest::rstest]
    #[case(
        SamplingConfig::Divide(NonZeroU16::MIN),
        SamplingConfig::Divide(NonZeroU16::MIN)
    )]
    #[case(SamplingConfig::FreqNearest(Nearest(1. * Hz)), SamplingConfig::Freq(1. * Hz))]
    #[case(
        SamplingConfig::PeriodNearest(Nearest(Duration::from_micros(1))),
        SamplingConfig::Period(Duration::from_micros(1))
    )]
    #[case(SamplingConfig::FreqNearest(Nearest(1. * Hz)), SamplingConfig::FreqNearest(Nearest(1. * Hz)))]
    #[case(
        SamplingConfig::PeriodNearest(Nearest(Duration::from_micros(1))),
        SamplingConfig::PeriodNearest(Nearest(Duration::from_micros(1)))
    )]
    #[test]
    fn into_nearest(#[case] expect: SamplingConfig, #[case] config: SamplingConfig) {
        assert_eq!(expect, config.into_nearest());
    }

    #[rstest::rstest]
    #[case(true, SamplingConfig::FREQ_40K, SamplingConfig::FREQ_40K)]
    #[case(true, SamplingConfig::FREQ_40K, SamplingConfig::new(NonZeroU16::MIN))]
    #[case(true, SamplingConfig::FREQ_40K, SamplingConfig::new(40. * kHz))]
    #[case(
        true,
        SamplingConfig::FREQ_40K,
        SamplingConfig::new(core::time::Duration::from_micros(25))
    )]
    #[case(false, SamplingConfig::new(41. * kHz), SamplingConfig::new(41. * kHz))]
    #[test]
    fn partial_eq(#[case] expect: bool, #[case] lhs: SamplingConfig, #[case] rhs: SamplingConfig) {
        assert_eq!(expect, lhs == rhs);
    }

    #[rstest::rstest]
    #[case("SamplingConfig::Divide(1)", SamplingConfig::Divide(NonZeroU16::MIN))]
    #[case("SamplingConfig::Freq(1 Hz)", SamplingConfig::Freq(1. * Hz))]
    #[case(
        "SamplingConfig::Period(1µs)",
        SamplingConfig::Period(Duration::from_micros(1))
    )]
    #[case("SamplingConfig::FreqNearest(Nearest(1 Hz))", SamplingConfig::FreqNearest(Nearest(1. * Hz)))]
    #[case(
        "SamplingConfig::PeriodNearest(Nearest(1µs))",
        SamplingConfig::PeriodNearest(Nearest(Duration::from_micros(1)))
    )]
    #[test]
    fn debug(#[case] expect: &str, #[case] config: SamplingConfig) {
        assert_eq!(expect, format!("{config:?}"));
    }
}
