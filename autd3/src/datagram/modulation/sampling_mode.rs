use autd3_driver::{
    defined::{Freq, Frequency, Hz, ULTRASOUND_FREQ},
    derive::SamplingConfig,
    error::AUTDInternalError,
    firmware::fpga::MOD_BUF_SIZE_MAX,
    utils::float::is_integer,
};
use num::integer::gcd;
use std::fmt::Debug;

/// A trait for sampling mode.
pub trait SamplingMode: Clone + Sync + Debug {
    /// Frequency type
    type T: Frequency;
    /// Calculate the frequency to be output.
    fn freq(freq: Self::T, _sampling_config: SamplingConfig) -> Self::T {
        freq
    }
    /// Validate the frequency.
    fn validate(
        freq: Self::T,
        sampling_config: SamplingConfig,
    ) -> Result<(u64, u64), AUTDInternalError>;
}

/// Exact frequency sampling mode with integer number.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExactFreq;

impl SamplingMode for ExactFreq {
    type T = Freq<u32>;
    fn validate(
        freq: Freq<u32>,
        sampling_config: SamplingConfig,
    ) -> Result<(u64, u64), AUTDInternalError> {
        if freq.hz() as f32 >= sampling_config.freq().hz() / 2. {
            return Err(AUTDInternalError::ModulationError(format!(
                "Frequency ({:?}) is equal to or greater than the Nyquist frequency ({:?})",
                freq,
                sampling_config.freq() / 2.
            )));
        }
        if freq.hz() == 0 {
            return Err(AUTDInternalError::ModulationError(
                "Frequency must not be zero. If intentional, Use `Static` instead.".to_string(),
            ));
        }

        let fd = freq.hz() as u64 * sampling_config.division() as u64;
        let fs = ULTRASOUND_FREQ.hz() as u64;

        let k = gcd(fs, fd);
        Ok((fs / k, fd / k))
    }
}

/// Exact frequency sampling mode with floating point number.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExactFreqFloat;

impl SamplingMode for ExactFreqFloat {
    type T = Freq<f32>;
    fn validate(
        freq: Freq<f32>,
        sampling_config: SamplingConfig,
    ) -> Result<(u64, u64), AUTDInternalError> {
        if freq.hz() < 0. || freq.hz().is_nan() {
            return Err(AUTDInternalError::ModulationError(format!(
                "Frequency ({:?}) must be valid positive value",
                freq
            )));
        }
        if freq.hz() == 0. {
            return Err(AUTDInternalError::ModulationError(
                "Frequency must not be zero. If intentional, Use `Static` instead.".to_string(),
            ));
        }
        if freq.hz() >= sampling_config.freq().hz() / 2. {
            return Err(AUTDInternalError::ModulationError(format!(
                "Frequency ({:?}) is equal to or greater than the Nyquist frequency ({:?})",
                freq,
                sampling_config.freq() / 2.
            )));
        }
        let fd = freq.hz() as f64 * sampling_config.division() as f64;

        for n in (ULTRASOUND_FREQ.hz() as f64 / fd).floor() as u32..=MOD_BUF_SIZE_MAX as u32 {
            if !is_integer(fd * n as f64) {
                continue;
            }
            let fnd = (fd * n as f64) as u64;
            let fs = ULTRASOUND_FREQ.hz() as u64;
            if fnd % fs != 0 {
                continue;
            }
            let k = fnd / fs;
            return Ok((n as _, k as _));
        }
        Err(AUTDInternalError::ModulationError(format!(
            "Frequency ({:?}) cannot be output with the sampling config ({:?}).",
            freq, sampling_config
        )))
    }
}

/// Nearest frequency sampling mode.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NearestFreq;

impl SamplingMode for NearestFreq {
    type T = Freq<f32>;
    fn freq(freq: Self::T, sampling_config: SamplingConfig) -> Self::T {
        let freq_min = sampling_config.freq().hz() / MOD_BUF_SIZE_MAX as f32;
        let freq_max = sampling_config.freq().hz() / 2.;
        freq.hz().clamp(freq_min, freq_max) * Hz
    }

    fn validate(
        freq: Freq<f32>,
        sampling_config: SamplingConfig,
    ) -> Result<(u64, u64), AUTDInternalError> {
        let freq = Self::freq(freq, sampling_config);
        if freq.hz().is_nan() {
            return Err(AUTDInternalError::ModulationError(format!(
                "Frequency ({:?}) must be valid value",
                freq
            )));
        }
        Ok(((sampling_config.freq().hz() / freq.hz()).round() as u64, 1))
    }
}

/// A trait for sampling mode inference.
pub trait SamplingModeInference: Copy + Clone + std::fmt::Debug + PartialEq {
    /// Inferred sampling mode.
    type T: SamplingMode<T = Self>;
}

impl SamplingModeInference for Freq<u32> {
    type T = ExactFreq;
}

impl SamplingModeInference for Freq<f32> {
    type T = ExactFreqFloat;
}

#[cfg(test)]
mod tests {

    use autd3_driver::defined::Hz;

    use super::*;

    #[rstest::rstest]
    #[test]
    #[case(1.2207031 * Hz, 1. * Hz, SamplingConfig::FREQ_40K)]
    #[case(1.2207031 * Hz, 1.2207031 * Hz, SamplingConfig::FREQ_40K)]
    #[case(20000. * Hz, 20000. * Hz, SamplingConfig::FREQ_40K)]
    #[case(20000. * Hz, 40000. * Hz, SamplingConfig::FREQ_40K)]
    fn nearest_freq_clamp(
        #[case] expect: Freq<f32>,
        #[case] freq: Freq<f32>,
        #[case] sampling_config: SamplingConfig,
    ) {
        assert_eq!(expect, NearestFreq::freq(freq, sampling_config));
    }
}
