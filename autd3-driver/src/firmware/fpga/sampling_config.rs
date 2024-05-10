use std::fmt::Debug;

use crate::{
    defined::{Freq, Hz},
    error::AUTDInternalError,
    firmware::fpga::{SAMPLING_FREQ_DIV_MAX, SAMPLING_FREQ_DIV_MIN},
};

use super::ULTRASOUND_PERIOD;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SamplingConfig {
    Freq(Freq<u32>),
    FreqNearest(Freq<f64>),
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

fn freq_min_raw(base_freq: Freq<u32>) -> Freq<f64> {
    (base_freq.hz() as f64 / div_max_raw() as f64) * Hz
}
fn freq_max_raw(base_freq: Freq<u32>) -> Freq<f64> {
    (base_freq.hz() as f64 / div_min_raw() as f64) * Hz
}

impl SamplingConfig {
    pub const DISABLE: Self = Self::DivisionRaw(0xFFFFFFFF);

    fn division_from_freq_nearest(
        f: Freq<f64>,
        base_freq: Freq<u32>,
    ) -> Result<u32, AUTDInternalError> {
        if !(freq_min_raw(base_freq).hz()..=freq_max_raw(base_freq).hz()).contains(&f.hz()) {
            Err(AUTDInternalError::SamplingFreqOutOfRange(
                f,
                freq_min_raw(base_freq),
                freq_max_raw(base_freq),
            ))
        } else {
            Ok((base_freq.hz() as f64 / f.hz()) as _)
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

    pub fn division(&self, ultrasound_freq: Freq<u32>) -> Result<u32, AUTDInternalError> {
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
                if (ultrasound_freq.hz() % f.hz()) != 0 {
                    return Err(AUTDInternalError::SamplingFreqInvalid(f, ultrasound_freq));
                }
                Self::division_from_freq_nearest((f.hz() as f64) * Hz, base_freq)
            }
            Self::FreqNearest(f) => Self::division_from_freq_nearest(f, base_freq),
        }
    }

    pub fn freq(&self, ultrasound_freq: Freq<u32>) -> Result<Freq<f64>, AUTDInternalError> {
        self.division(ultrasound_freq)
            .map(|d| (ultrasound_freq.hz() * ULTRASOUND_PERIOD) as f64 / d as f64 * Hz)
    }
}

impl std::fmt::Display for SamplingConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Freq(freq) => {
                write!(f, "{}", freq)
            }
            Self::FreqNearest(freq) => {
                write!(f, "{}", freq)
            }
            Self::Division(d) | Self::DivisionRaw(d) => {
                write!(f, "Division({})", d)
            }
        }
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

    #[rstest::rstest]
    #[test]
    #[case::min(Ok(div_min()), div_min(), 40000 * Hz)]
    #[case::max(Ok(div_max()), div_max(), 40000 * Hz)]
    #[case::invalid(
        Err(AUTDInternalError::SamplingFreqDivInvalid(
            div_min() + 1
        )),
        div_min() + 1,
        40000 * Hz
    )]
    #[case::out_of_range(
        Err(AUTDInternalError::SamplingFreqDivOutOfRange(0, div_min_raw(), div_max_raw())),
        0,
        40000 * Hz
    )]
    #[case::min(Ok(div_min()), div_min(), 80000 * Hz)]
    #[case::max(Ok(div_max()), div_max(), 80000 * Hz)]
    #[case::invalid(
        Err(AUTDInternalError::SamplingFreqDivInvalid(
            div_min() + 1
        )),
        div_min() + 1,
        80000 * Hz
    )]
    #[case::out_of_range(
        Err(AUTDInternalError::SamplingFreqDivOutOfRange(0, div_min_raw(), div_max_raw())),
        0,
        80000 * Hz
    )]
    fn division_from_division(
        #[case] expected: Result<u32, AUTDInternalError>,
        #[case] freq_div: u32,
        #[case] ultrasound_freq: Freq<u32>,
    ) {
        assert_eq!(
            expected,
            SamplingConfig::Division(freq_div).division(ultrasound_freq)
        );
    }

    #[rstest::rstest]
    #[test]
    #[case::min(Ok(div_min_raw()), div_min_raw(), 40000 * Hz)]
    #[case::max(Ok(div_max_raw()), div_max_raw(), 40000 * Hz)]
    #[case::invalid(
        Ok(div_min_raw() + 1),
        div_min_raw() + 1,
        40000 * Hz
    )]
    #[case::out_of_range(
        Err(AUTDInternalError::SamplingFreqDivOutOfRange(0, div_min_raw(), div_max_raw())),
        0,
        40000 * Hz
    )]
    #[case::min(Ok(div_min_raw()), div_min_raw(), 80000 * Hz)]
    #[case::max(Ok(div_max_raw()), div_max_raw(), 80000 * Hz)]
    #[case::invalid(
        Ok(div_min_raw() + 1),
        div_min_raw() + 1,
        80000 * Hz
    )]
    #[case::out_of_range(
        Err(AUTDInternalError::SamplingFreqDivOutOfRange(0, div_min_raw(), div_max_raw())),
        0,
        80000 * Hz
    )]
    fn from_division_raw(
        #[case] expected: Result<u32, AUTDInternalError>,
        #[case] freq_div: u32,
        #[case] ultrasound_freq: Freq<u32>,
    ) {
        assert_eq!(
            expected,
            SamplingConfig::DivisionRaw(freq_div).division(ultrasound_freq)
        );
    }

    #[rstest::rstest]
    #[test]
    #[case::min(Ok(20480000), freq_min(), 40000 * Hz)]
    #[case::max(Ok(512), freq_max(20480000 * Hz), 40000 * Hz)]
    #[case(Err(AUTDInternalError::SamplingFreqInvalid(512*Hz, 40000 * Hz)), 512*Hz, 40000 * Hz)]
    #[case::not_supported_max(
        Err(AUTDInternalError::SamplingFreqInvalid(
            freq_max(20480000 * Hz) - 1 * Hz,
            40000 * Hz
        )),
        freq_max(20480000 * Hz) - 1 * Hz,
        40000 * Hz
    )]
    #[case::out_of_range_max(
        Err(AUTDInternalError::SamplingFreqInvalid(
            freq_max(20480000 * Hz) * 2,
            40000 * Hz
        )),
        freq_max(20480000 * Hz) * 2,
        40000 * Hz
    )]
    #[case::min(Ok(80000*512), freq_min(), 80000 * Hz)]
    #[case::max(Ok(512), freq_max(80000*512 * Hz), 80000 * Hz)]
    #[case(Err(AUTDInternalError::SamplingFreqInvalid(512*Hz, 80000 * Hz)), 512*Hz, 80000 * Hz)]
    #[case::not_supported_max(
        Err(AUTDInternalError::SamplingFreqInvalid(
            freq_max(80000*512 * Hz) - 1*Hz,
            80000 * Hz
        )),
        freq_max(80000*512 * Hz) - 1*Hz,
        80000 * Hz
    )]
    #[case::out_of_range_max(
        Err(AUTDInternalError::SamplingFreqInvalid(
            freq_max(80000*512 * Hz) * 2,
            80000 * Hz
        )),
        freq_max(80000*512 * Hz) * 2,
        80000 * Hz
    )]
    fn from_freq(
        #[case] expected: Result<u32, AUTDInternalError>,
        #[case] freq: Freq<u32>,
        #[case] ultrasound_freq: Freq<u32>,
    ) {
        assert_eq!(
            expected,
            SamplingConfig::Freq(freq).division(ultrasound_freq)
        );
    }

    #[rstest::rstest]
    #[test]
    #[case::min(Ok(0xFFFFFFFF), freq_min_raw(20480000 * Hz), 40000 * Hz)]
    #[case::max(Ok(512), freq_max_raw(20480000 * Hz), 40000 * Hz)]
    #[case(Ok(40000), 512.*Hz, 40000 * Hz)]
    #[case::not_supported_max(
        Ok(512),
        (freq_max(20480000 * Hz).hz() as f64 - 1.)*Hz,
        40000 * Hz
    )]
    #[case::out_of_range_min(
        Err(AUTDInternalError::SamplingFreqOutOfRange(
            freq_min_raw(20480000 * Hz) - f64::MIN * Hz,
            freq_min_raw(20480000 * Hz),
            freq_max_raw(20480000 * Hz)
        )),
        freq_min_raw(20480000 * Hz) - f64::MIN*Hz,
        40000 * Hz
    )]
    #[case::out_of_range_max(
        Err(AUTDInternalError::SamplingFreqOutOfRange(
            freq_max_raw(20480000 * Hz) + f64::MIN * Hz,
            freq_min_raw(20480000 * Hz),
            freq_max_raw(20480000 * Hz)
        )),
        freq_max_raw(20480000 * Hz) + f64::MIN*Hz,
        40000 * Hz
    )]
    #[case::min(Ok(0xFFFFFFFF), freq_min_raw(80000*512*Hz), 80000 * Hz)]
    #[case::max(Ok(512), freq_max_raw(80000*512*Hz), 80000 * Hz)]
    #[case(Ok(80000), 512.*Hz, 80000 * Hz)]
    #[case::not_supported_max(
        Ok(512),
        (freq_max(80000*512*Hz).hz() as f64 - 1.)*Hz,
        80000 * Hz
    )]
    #[case::out_of_range_min(
        Err(AUTDInternalError::SamplingFreqOutOfRange(
            freq_min_raw(80000*512*Hz) - f64::MIN * Hz,
            freq_min_raw(80000*512*Hz),
            freq_max_raw(80000*512*Hz)
        )),
        freq_min_raw(20480000*Hz) - f64::MIN*Hz,
        80000 * Hz
    )]
    #[case::out_of_range_max(
        Err(AUTDInternalError::SamplingFreqOutOfRange(
            freq_max_raw(80000*512*Hz) + f64::MIN * Hz,
            freq_min_raw(80000*512*Hz),
            freq_max_raw(80000*512*Hz)
        )),
        freq_max_raw(80000*512*Hz) + f64::MIN*Hz,
        80000 * Hz
    )]
    fn from_freq_nearest(
        #[case] expected: Result<u32, AUTDInternalError>,
        #[case] freq: Freq<f64>,
        #[case] ultrasound_freq: Freq<u32>,
    ) {
        assert_eq!(
            expected,
            SamplingConfig::FreqNearest(freq).division(ultrasound_freq)
        );
    }

    #[rstest::rstest]
    #[test]
    #[case::freq(SamplingConfig::Freq(4000*Hz), "4000 Hz")]
    #[case::freq(SamplingConfig::FreqNearest(4000.*Hz), "4000 Hz")]
    #[case::div(SamplingConfig::Division(305419896), "Division(305419896)")]
    fn display(#[case] config: SamplingConfig, #[case] expected: &str) {
        assert_eq!(expected, config.to_string());
    }
}
