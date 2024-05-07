use std::fmt::Debug;

use crate::{
    error::AUTDInternalError,
    firmware::fpga::{SAMPLING_FREQ_DIV_MAX, SAMPLING_FREQ_DIV_MIN},
};

use super::ULTRASOUND_PERIOD;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SamplingConfig {
    Freq(u32),
    FreqNearest(f64),
    DivisionRaw(u32),
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

fn freq_min_raw(base_freq: u32) -> f64 {
    base_freq as f64 / div_max_raw() as f64
}
fn freq_max_raw(base_freq: u32) -> f64 {
    base_freq as f64 / div_min_raw() as f64
}

impl SamplingConfig {
    pub const DISABLE: Self = Self::DivisionRaw(0xFFFFFFFF);
    pub const FREQ_4K_HZ: Self = Self::Freq(4000);

    fn division_from_freq_nearest(f: f64, base_freq: u32) -> Result<u32, AUTDInternalError> {
        if !(freq_min_raw(base_freq)..=freq_max_raw(base_freq)).contains(&f) {
            Err(AUTDInternalError::SamplingFreqOutOfRange(
                f,
                freq_min_raw(base_freq),
                freq_max_raw(base_freq),
            ))
        } else {
            Ok((base_freq as f64 / f) as _)
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

    pub fn division(&self, ultrasound_freq: u32) -> Result<u32, AUTDInternalError> {
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
                if (ultrasound_freq % f) != 0 {
                    return Err(AUTDInternalError::SamplingFreqInvalid(f, ultrasound_freq));
                }
                Self::division_from_freq_nearest(f as _, base_freq)
            }
            Self::FreqNearest(f) => Self::division_from_freq_nearest(f, base_freq),
        }
    }

    pub fn freq(&self, ultrasound_freq: u32) -> Result<f64, AUTDInternalError> {
        self.division(ultrasound_freq)
            .map(|d| (ultrasound_freq * ULTRASOUND_PERIOD) as f64 / d as f64)
    }
}

impl std::fmt::Display for SamplingConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Freq(freq) => {
                write!(f, "{}Hz", freq)
            }
            Self::FreqNearest(freq) => {
                write!(f, "{}Hz", freq)
            }
            Self::Division(d) | Self::DivisionRaw(d) => {
                write!(f, "Division({})", d)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn div_max() -> u32 {
        (SAMPLING_FREQ_DIV_MAX / SAMPLING_FREQ_DIV_MIN) * SAMPLING_FREQ_DIV_MIN
    }
    const fn freq_min() -> u32 {
        1
    }
    const fn freq_max(base_freq: u32) -> u32 {
        base_freq / div_min()
    }

    #[rstest::rstest]
    #[test]
    #[case::min(Ok(div_min()), div_min(), 40000)]
    #[case::max(Ok(div_max()), div_max(), 40000)]
    #[case::invalid(
        Err(AUTDInternalError::SamplingFreqDivInvalid(
            div_min() + 1
        )),
        div_min() + 1,
        40000
    )]
    #[case::out_of_range(
        Err(AUTDInternalError::SamplingFreqDivOutOfRange(0, div_min_raw(), div_max_raw())),
        0,
        40000
    )]
    #[case::min(Ok(div_min()), div_min(), 80000)]
    #[case::max(Ok(div_max()), div_max(), 80000)]
    #[case::invalid(
        Err(AUTDInternalError::SamplingFreqDivInvalid(
            div_min() + 1
        )),
        div_min() + 1,
        80000
    )]
    #[case::out_of_range(
        Err(AUTDInternalError::SamplingFreqDivOutOfRange(0, div_min_raw(), div_max_raw())),
        0,
        80000
    )]
    fn division_from_division(
        #[case] expected: Result<u32, AUTDInternalError>,
        #[case] freq_div: u32,
        #[case] ultrasound_freq: u32,
    ) {
        assert_eq!(
            expected,
            SamplingConfig::Division(freq_div).division(ultrasound_freq)
        );
    }

    #[rstest::rstest]
    #[test]
    #[case::min(Ok(div_min_raw()), div_min_raw(), 40000)]
    #[case::max(Ok(div_max_raw()), div_max_raw(), 40000)]
    #[case::invalid(
        Ok(div_min_raw() + 1),
        div_min_raw() + 1,
        40000
    )]
    #[case::out_of_range(
        Err(AUTDInternalError::SamplingFreqDivOutOfRange(0, div_min_raw(), div_max_raw())),
        0,
        40000
    )]
    #[case::min(Ok(div_min_raw()), div_min_raw(), 80000)]
    #[case::max(Ok(div_max_raw()), div_max_raw(), 80000)]
    #[case::invalid(
        Ok(div_min_raw() + 1),
        div_min_raw() + 1,
        80000
    )]
    #[case::out_of_range(
        Err(AUTDInternalError::SamplingFreqDivOutOfRange(0, div_min_raw(), div_max_raw())),
        0,
        80000
    )]
    fn from_division_raw(
        #[case] expected: Result<u32, AUTDInternalError>,
        #[case] freq_div: u32,
        #[case] ultrasound_freq: u32,
    ) {
        assert_eq!(
            expected,
            SamplingConfig::DivisionRaw(freq_div).division(ultrasound_freq)
        );
    }

    #[rstest::rstest]
    #[test]
    #[case::min(Ok(20480000), freq_min(), 40000)]
    #[case::max(Ok(512), freq_max(20480000), 40000)]
    #[case(Err(AUTDInternalError::SamplingFreqInvalid(512, 40000)), 512, 40000)]
    #[case::not_supported_max(
        Err(AUTDInternalError::SamplingFreqInvalid(
            freq_max(20480000) - 1,
            40000
        )),
        freq_max(20480000) - 1,
        40000
    )]
    #[case::out_of_range_max(
        Err(AUTDInternalError::SamplingFreqInvalid(
            freq_max(20480000) * 2,
            40000
        )),
        freq_max(20480000) * 2,
        40000
    )]
    #[case::min(Ok(80000*512), freq_min(), 80000)]
    #[case::max(Ok(512), freq_max(80000*512), 80000)]
    #[case(Err(AUTDInternalError::SamplingFreqInvalid(512, 80000)), 512, 80000)]
    #[case::not_supported_max(
        Err(AUTDInternalError::SamplingFreqInvalid(
            freq_max(80000*512) - 1,
            80000
        )),
        freq_max(80000*512) - 1,
        80000
    )]
    #[case::out_of_range_max(
        Err(AUTDInternalError::SamplingFreqInvalid(
            freq_max(80000*512) * 2,
            80000
        )),
        freq_max(80000*512) * 2,
        80000
    )]
    fn from_freq(
        #[case] expected: Result<u32, AUTDInternalError>,
        #[case] freq: u32,
        #[case] ultrasound_freq: u32,
    ) {
        assert_eq!(
            expected,
            SamplingConfig::Freq(freq).division(ultrasound_freq)
        );
    }

    #[rstest::rstest]
    #[test]
    #[case::min(Ok(0xFFFFFFFF), freq_min_raw(20480000), 40000)]
    #[case::max(Ok(512), freq_max_raw(20480000), 40000)]
    #[case(Ok(40000), 512., 40000)]
    #[case::not_supported_max(
        Ok(512),
        freq_max(20480000) as f64 - 1.,
        40000
    )]
    #[case::out_of_range_min(
        Err(AUTDInternalError::SamplingFreqOutOfRange(
            freq_min_raw(20480000) - f64::MIN,
            freq_min_raw(20480000),
            freq_max_raw(20480000)
        )),
        freq_min_raw(20480000) - f64::MIN,
        40000
    )]
    #[case::out_of_range_max(
        Err(AUTDInternalError::SamplingFreqOutOfRange(
            freq_max_raw(20480000) + f64::MIN,
            freq_min_raw(20480000),
            freq_max_raw(20480000)
        )),
        freq_max_raw(20480000) + f64::MIN,
        40000
    )]
    #[case::min(Ok(0xFFFFFFFF), freq_min_raw(80000*512), 80000)]
    #[case::max(Ok(512), freq_max_raw(80000*512), 80000)]
    #[case(Ok(80000), 512., 80000)]
    #[case::not_supported_max(
        Ok(512),
        freq_max(80000*512) as f64 - 1.,
        80000
    )]
    #[case::out_of_range_min(
        Err(AUTDInternalError::SamplingFreqOutOfRange(
            freq_min_raw(80000*512) - f64::MIN,
            freq_min_raw(80000*512),
            freq_max_raw(80000*512)
        )),
        freq_min_raw(20480000) - f64::MIN,
        80000
    )]
    #[case::out_of_range_max(
        Err(AUTDInternalError::SamplingFreqOutOfRange(
            freq_max_raw(80000*512) + f64::MIN,
            freq_min_raw(80000*512),
            freq_max_raw(80000*512)
        )),
        freq_max_raw(80000*512) + f64::MIN,
        80000
    )]
    fn from_freq_nearest(
        #[case] expected: Result<u32, AUTDInternalError>,
        #[case] freq: f64,
        #[case] ultrasound_freq: u32,
    ) {
        assert_eq!(
            expected,
            SamplingConfig::FreqNearest(freq).division(ultrasound_freq)
        );
    }

    #[rstest::rstest]
    #[test]
    #[case::freq(SamplingConfig::Freq(4000), "4000Hz")]
    #[case::freq(SamplingConfig::FreqNearest(4000.), "4000Hz")]
    #[case::div(SamplingConfig::Division(305419896), "Division(305419896)")]
    fn display(#[case] config: SamplingConfig, #[case] expected: &str) {
        assert_eq!(expected, config.to_string());
    }
}
