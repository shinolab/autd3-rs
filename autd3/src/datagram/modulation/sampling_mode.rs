use autd3_core::{
    derive::{ModulationError, SamplingConfig},
    utils::float::is_integer,
};
use autd3_driver::{
    defined::{Freq, Hz, ULTRASOUND_FREQ},
    firmware::fpga::MOD_BUF_SIZE_MAX,
};
use num::integer::gcd;
use std::fmt::Debug;

/// Nearest frequency type.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Nearest(pub Freq<f32>);

/// A enum for sampling mode.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SamplingMode {
    /// Exact frequency sampling mode with integer number.
    ExactFreq(Freq<u32>),
    /// Exact frequency sampling mode with floating point number.
    ExactFreqFloat(Freq<f32>),
    /// Nearest frequency sampling mode.
    NearestFreq(Freq<f32>),
}

impl SamplingMode {
    pub(crate) fn validate(
        self,
        sampling_config: SamplingConfig,
    ) -> Result<(u64, u64), ModulationError> {
        match self {
            SamplingMode::ExactFreq(freq) => Self::validate_exact(freq, sampling_config),
            SamplingMode::ExactFreqFloat(freq) => Self::validate_exact_f(freq, sampling_config),
            SamplingMode::NearestFreq(freq) => Self::validate_nearest(freq, sampling_config),
        }
    }
}

impl SamplingMode {
    fn validate_exact(
        freq: Freq<u32>,
        sampling_config: SamplingConfig,
    ) -> Result<(u64, u64), ModulationError> {
        if freq.hz() as f32 >= sampling_config.freq()?.hz() / 2. {
            return Err(ModulationError::new(format!(
                "Frequency ({:?}) is equal to or greater than the Nyquist frequency ({:?})",
                freq,
                sampling_config.freq()? / 2.
            )));
        }
        if freq.hz() == 0 {
            return Err(ModulationError::new(
                "Frequency must not be zero. If intentional, Use `Static` instead.",
            ));
        }

        let fd = freq.hz() as u64 * sampling_config.divide()? as u64;
        let fs = ULTRASOUND_FREQ.hz() as u64;

        let k = gcd(fs, fd);
        Ok((fs / k, fd / k))
    }
}

impl SamplingMode {
    fn validate_exact_f(
        freq: Freq<f32>,
        sampling_config: SamplingConfig,
    ) -> Result<(u64, u64), ModulationError> {
        if freq.hz() < 0. || freq.hz().is_nan() {
            return Err(ModulationError::new(format!(
                "Frequency ({:?}) must be valid positive value",
                freq
            )));
        }
        if freq.hz() == 0. {
            return Err(ModulationError::new(
                "Frequency must not be zero. If intentional, Use `Static` instead.",
            ));
        }
        if freq.hz() >= sampling_config.freq()?.hz() / 2. {
            return Err(ModulationError::new(format!(
                "Frequency ({:?}) is equal to or greater than the Nyquist frequency ({:?})",
                freq,
                sampling_config.freq()? / 2.
            )));
        }
        let fd = freq.hz() as f64 * sampling_config.divide()? as f64;

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
        Err(ModulationError::new(format!(
            "Frequency ({:?}) cannot be output with the sampling config ({:?}).",
            freq, sampling_config
        )))
    }
}

impl SamplingMode {
    fn freq_nearest(
        freq: Freq<f32>,
        sampling_config: SamplingConfig,
    ) -> Result<Freq<f32>, ModulationError> {
        let freq_min = sampling_config.freq()?.hz() / MOD_BUF_SIZE_MAX as f32;
        let freq_max = sampling_config.freq()?.hz() / 2.;
        Ok(freq.hz().clamp(freq_min, freq_max) * Hz)
    }

    fn validate_nearest(
        freq: Freq<f32>,
        sampling_config: SamplingConfig,
    ) -> Result<(u64, u64), ModulationError> {
        let freq = Self::freq_nearest(freq, sampling_config)?;
        if freq.hz().is_nan() {
            return Err(ModulationError::new(format!(
                "Frequency ({:?}) must be valid value",
                freq
            )));
        }
        Ok(((sampling_config.freq()?.hz() / freq.hz()).round() as u64, 1))
    }
}

impl From<Freq<u32>> for SamplingMode {
    fn from(val: Freq<u32>) -> Self {
        SamplingMode::ExactFreq(val)
    }
}

impl From<Freq<f32>> for SamplingMode {
    fn from(val: Freq<f32>) -> Self {
        SamplingMode::ExactFreqFloat(val)
    }
}

impl From<Nearest> for SamplingMode {
    fn from(val: Nearest) -> Self {
        SamplingMode::NearestFreq(val.0)
    }
}

#[cfg(test)]
mod tests {

    use autd3_driver::defined::Hz;

    use super::*;

    #[rstest::rstest]
    #[test]
    #[case(0.61035156 * Hz, 0.5 * Hz, SamplingConfig::FREQ_40K)]
    #[case(0.61035156 * Hz, 0.61035156 * Hz, SamplingConfig::FREQ_40K)]
    #[case(20000. * Hz, 20000. * Hz, SamplingConfig::FREQ_40K)]
    #[case(20000. * Hz, 40000. * Hz, SamplingConfig::FREQ_40K)]
    fn nearest_freq_clamp(
        #[case] expect: Freq<f32>,
        #[case] freq: Freq<f32>,
        #[case] sampling_config: SamplingConfig,
    ) {
        assert_eq!(
            Ok(expect),
            SamplingMode::freq_nearest(freq, sampling_config)
        );
    }
}
