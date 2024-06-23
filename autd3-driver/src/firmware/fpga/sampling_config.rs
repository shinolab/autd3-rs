use std::{fmt::Debug, time::Duration};

use crate::{
    defined::{Freq, Hz},
    error::AUTDInternalError,
    firmware::fpga::{SAMPLING_FREQ_DIV_MAX, SAMPLING_FREQ_DIV_MIN},
    get_ultrasound_freq,
};

use derive_more::Display;

use super::ULTRASOUND_PERIOD;

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
    #[display(fmt = "DivisionRaw({})", _0)]
    DivisionRaw(u32),
    #[display(fmt = "Division({})", _0)]
    Division(u32),
}

const fn div_min_raw() -> u32 {
    SAMPLING_FREQ_DIV_MIN
}
const fn div_max_raw() -> u32 {
    SAMPLING_FREQ_DIV_MAX
}
const fn div_min() -> u32 {
    SAMPLING_FREQ_DIV_MIN
}

fn freq_min_raw(base_freq: Freq<u32>) -> Freq<f32> {
    (base_freq.hz() as f32 / div_max_raw() as f32) * Hz
}
fn freq_max_raw(base_freq: Freq<u32>) -> Freq<f32> {
    (base_freq.hz() as f32 / div_min_raw() as f32) * Hz
}

const fn period_min_raw(base_freq: Freq<u32>) -> Duration {
    Duration::from_nanos((div_min_raw() as u128 * NANOSEC / base_freq.hz() as u128) as u64)
}
const fn period_max_raw(base_freq: Freq<u32>) -> Duration {
    Duration::from_nanos((div_max_raw() as u128 * NANOSEC / base_freq.hz() as u128) as u64)
}

impl SamplingConfig {
    pub const DISABLE: Self = Self::DivisionRaw(0xFFFFFFFF);

    fn division_from_freq_nearest(
        f: Freq<f32>,
        base_freq: Freq<u32>,
    ) -> Result<u32, AUTDInternalError> {
        if !(freq_min_raw(base_freq).hz()..=freq_max_raw(base_freq).hz()).contains(&f.hz()) {
            Err(AUTDInternalError::SamplingFreqOutOfRange(
                f,
                freq_min_raw(base_freq),
                freq_max_raw(base_freq),
            ))
        } else {
            Ok((base_freq.hz() as f32 / f.hz()) as _)
        }
    }

    fn division_from_period_nearest(
        p: Duration,
        base_freq: Freq<u32>,
    ) -> Result<u32, AUTDInternalError> {
        if !(period_min_raw(base_freq)..=period_max_raw(base_freq)).contains(&p) {
            Err(AUTDInternalError::SamplingPeriodOutOfRange(
                p,
                period_min_raw(base_freq),
                period_max_raw(base_freq),
            ))
        } else {
            let k = (p.as_nanos() * base_freq.hz() as u128) / NANOSEC;
            Ok(k as _)
        }
    }

    fn division_from_division_raw(d: u32) -> Result<u32, AUTDInternalError> {
        if !(div_min_raw()..=div_max_raw()).contains(&d) {
            Err(AUTDInternalError::SamplingFreqDivOutOfRange(
                d,
                div_min_raw(),
                div_max_raw(),
            ))
        } else {
            Ok(d)
        }
    }

    pub fn division(&self) -> Result<u32, AUTDInternalError> {
        let ultrasound_freq = get_ultrasound_freq();
        let base_freq = ultrasound_freq * ULTRASOUND_PERIOD;
        match *self {
            Self::Division(div) => {
                if div % div_min() != 0 {
                    return Err(AUTDInternalError::SamplingFreqDivInvalid(div));
                }
                Self::division_from_division_raw(div)
            }
            Self::DivisionRaw(div) => Self::division_from_division_raw(div),
            Self::Freq(f) => {
                if ultrasound_freq.hz() % f.hz() != 0 {
                    return Err(AUTDInternalError::SamplingFreqInvalid(f, ultrasound_freq));
                }
                Self::division_from_freq_nearest((f.hz() as f32) * Hz, base_freq)
            }
            Self::FreqNearest(f) => Self::division_from_freq_nearest(f, base_freq),
            Self::Period(p) => {
                let k = p.as_nanos() * ultrasound_freq.hz() as u128;
                if k % NANOSEC != 0 {
                    return Err(AUTDInternalError::SamplingPeriodInvalid(p));
                }
                Self::division_from_period_nearest(p, base_freq)
            }
            Self::PeriodNearest(p) => Self::division_from_period_nearest(p, base_freq),
        }
    }

    pub fn freq(&self) -> Result<Freq<f32>, AUTDInternalError> {
        let ultrasound_freq = get_ultrasound_freq();
        self.division()
            .map(|d| (ultrasound_freq.hz() * ULTRASOUND_PERIOD) as f32 / d as f32 * Hz)
    }

    pub fn period(&self) -> Result<Duration, AUTDInternalError> {
        self.division().map(|d| {
            Duration::from_nanos(
                (d as u128 * NANOSEC / (get_ultrasound_freq() * ULTRASOUND_PERIOD).hz() as u128)
                    as u64,
            )
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

    const fn div_max() -> u32 {
        (SAMPLING_FREQ_DIV_MAX / SAMPLING_FREQ_DIV_MIN) * SAMPLING_FREQ_DIV_MIN
    }
    fn freq_min() -> Freq<u32> {
        1 * Hz
    }
    fn freq_max(base_freq: Freq<u32>) -> Freq<u32> {
        base_freq / div_min()
    }
    const fn period_min() -> Duration {
        Duration::from_micros(25)
    }
    const fn period_max(base_freq: Freq<u32>) -> Duration {
        Duration::from_nanos((div_max() as u128 * NANOSEC / base_freq.hz() as u128) as u64)
    }

    #[rstest::rstest]
    #[test]
    #[case::min(Ok(div_min()), div_min())]
    #[case::max(Ok(div_max()), div_max())]
    #[case::invalid(
        Err(AUTDInternalError::SamplingFreqDivInvalid(
            div_min() + 1
        )),
        div_min() + 1
    )]
    #[case::out_of_range(
        Err(AUTDInternalError::SamplingFreqDivOutOfRange(0, div_min_raw(), div_max_raw())),
        0
    )]
    fn division_from_division(
        #[case] expected: Result<u32, AUTDInternalError>,
        #[case] freq_div: u32,
    ) {
        assert_eq!(expected, SamplingConfig::Division(freq_div).division());
    }

    #[rstest::rstest]
    #[test]
    #[case(Ok(Duration::from_micros(25)), SamplingConfig::Division(512))]
    #[case(Ok(Duration::from_micros(25)), SamplingConfig::Freq(40000*Hz))]
    #[case(
        Ok(Duration::from_micros(25)),
        SamplingConfig::Period(Duration::from_micros(25))
    )]
    #[case(
        Err(AUTDInternalError::SamplingFreqDivInvalid(513)),
        SamplingConfig::Division(513)
    )]
    fn period(
        #[case] expected: Result<Duration, AUTDInternalError>,
        #[case] config: SamplingConfig,
    ) {
        assert_eq!(expected, config.period());
    }

    #[rstest::rstest]
    #[test]
    #[case::min(Ok(div_min_raw()), div_min_raw())]
    #[case::max(Ok(div_max_raw()), div_max_raw())]
    #[case::invalid(
        Ok(div_min_raw() + 1),
        div_min_raw() + 1,
    )]
    #[case::out_of_range(
        Err(AUTDInternalError::SamplingFreqDivOutOfRange(0, div_min_raw(), div_max_raw())),
        0
    )]
    fn from_division_raw(#[case] expected: Result<u32, AUTDInternalError>, #[case] freq_div: u32) {
        assert_eq!(expected, SamplingConfig::DivisionRaw(freq_div).division());
    }

    #[rstest::rstest]
    #[test]
    #[case::min(Ok(20480000), freq_min())]
    #[case::max(Ok(512), freq_max(20480000 * Hz))]
    #[case(Err(AUTDInternalError::SamplingFreqInvalid(512*Hz, 40000 * Hz)), 512*Hz)]
    #[case::not_supported_max(
        Err(AUTDInternalError::SamplingFreqInvalid(
            freq_max(20480000 * Hz) - 1 * Hz,
            40000 * Hz
        )),
        freq_max(20480000 * Hz) - 1 * Hz,
    )]
    #[case::out_of_range_max(
        Err(AUTDInternalError::SamplingFreqInvalid(
            freq_max(20480000 * Hz) * 2,
            40000 * Hz
        )),
        freq_max(20480000 * Hz) * 2,
    )]
    fn from_freq(#[case] expected: Result<u32, AUTDInternalError>, #[case] freq: Freq<u32>) {
        assert_eq!(expected, SamplingConfig::Freq(freq).division());
    }

    #[rstest::rstest]
    #[test]
    #[case::min(Ok(0xFFFFFFFF), freq_min_raw(20480000 * Hz))]
    #[case::max(Ok(512), freq_max_raw(20480000 * Hz))]
    #[case(Ok(40000), 512.*Hz)]
    #[case::not_supported_max(
        Ok(512),
        (freq_max(20480000 * Hz).hz() as f32 - 1.)*Hz,
    )]
    #[case::out_of_range_min(
        Err(AUTDInternalError::SamplingFreqOutOfRange(
            freq_min_raw(20480000 * Hz) - f32::MIN * Hz,
            freq_min_raw(20480000 * Hz),
            freq_max_raw(20480000 * Hz)
        )),
        freq_min_raw(20480000 * Hz) - f32::MIN*Hz,
    )]
    #[case::out_of_range_max(
        Err(AUTDInternalError::SamplingFreqOutOfRange(
            freq_max_raw(20480000 * Hz) + f32::MIN * Hz,
            freq_min_raw(20480000 * Hz),
            freq_max_raw(20480000 * Hz)
        )),
        freq_max_raw(20480000 * Hz) + f32::MIN*Hz,
    )]
    fn from_freq_nearest(
        #[case] expected: Result<u32, AUTDInternalError>,
        #[case] freq: Freq<f32>,
    ) {
        assert_eq!(expected, SamplingConfig::FreqNearest(freq).division());
    }

    #[rstest::rstest]
    #[test]
    #[case::min(Ok(512), period_min())]
    #[case::max(Ok(4294966784), period_max(20480000 * Hz))]
    #[case(
        Err(AUTDInternalError::SamplingPeriodInvalid(Duration::from_micros(26))),
        Duration::from_micros(26)
    )]
    #[case::not_supported_max(
        Err(AUTDInternalError::SamplingPeriodInvalid(
            period_max(20480000 * Hz) - Duration::from_nanos(1)
        )),
        period_max(20480000 * Hz) - Duration::from_nanos(1),
    )]
    #[case::out_of_range_max(
        Err(AUTDInternalError::SamplingPeriodOutOfRange(
            period_max(20480000 * Hz) * 2,
            period_min(),
            period_max_raw(20480000 * Hz)
        )),
        period_max(20480000 * Hz) * 2,
    )]
    fn from_period(#[case] expected: Result<u32, AUTDInternalError>, #[case] period: Duration) {
        assert_eq!(expected, SamplingConfig::from(period).division());
    }

    #[rstest::rstest]
    #[test]
    #[case::min(Ok(512), period_min_raw(20480000 * Hz))]
    #[case::max(Ok(4294967294), period_max_raw(20480000 * Hz))]
    #[case(Ok(532), Duration::from_micros(26))]
    #[case::not_supported_max(
        Ok(4294966783),
        period_max(20480000 * Hz) - Duration::from_nanos(1),
    )]
    #[case::out_of_range_min(
        Err(AUTDInternalError::SamplingPeriodOutOfRange(
            period_min_raw(20480000 * Hz) - Duration::from_nanos(1),
            period_min_raw(20480000 * Hz),
            period_max_raw(20480000 * Hz)
        )),
        period_min_raw(20480000 * Hz) - Duration::from_nanos(1),
    )]
    #[case::out_of_range_max(
        Err(AUTDInternalError::SamplingPeriodOutOfRange(
            period_max(20480000 * Hz) * 2,
            period_min_raw(20480000 * Hz),
            period_max_raw(20480000 * Hz)
        )),
        period_max(20480000 * Hz) * 2,
    )]
    fn from_period_nearest(
        #[case] expected: Result<u32, AUTDInternalError>,
        #[case] period: Duration,
    ) {
        assert_eq!(expected, SamplingConfig::PeriodNearest(period).division());
    }

    #[rstest::rstest]
    #[test]
    #[case::freq(SamplingConfig::Freq(4000*Hz), "4000 Hz")]
    #[case::freq(SamplingConfig::FreqNearest(4000.*Hz), "4000 Hz")]
    #[case::div(SamplingConfig::Division(305419896), "Division(305419896)")]
    #[case::div(SamplingConfig::DivisionRaw(305419896), "DivisionRaw(305419896)")]
    #[case::div(SamplingConfig::Period(Duration::from_micros(25)), "25µs")]
    #[case::div(SamplingConfig::PeriodNearest(Duration::from_micros(25)), "25µs")]
    fn display(#[case] config: SamplingConfig, #[case] expected: &str) {
        assert_eq!(expected, config.to_string());
    }
}
