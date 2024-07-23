use autd3_driver::{
    defined::{Freq, Frequency, ULTRASOUND_FREQ},
    derive::SamplingConfig,
    error::AUTDInternalError,
    firmware::fpga::MOD_BUF_SIZE_MAX,
    utils::float::is_integer,
};
use num::integer::gcd;
use std::fmt::Debug;

pub trait SamplingMode: Clone + Sync + Debug {
    type T: Frequency;
    fn validate(
        freq: Self::T,
        sampling_config: SamplingConfig,
    ) -> Result<(u64, u64), AUTDInternalError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExactFreq;

impl SamplingMode for ExactFreq {
    type T = Freq<u32>;
    fn validate(
        freq: Freq<u32>,
        sampling_config: SamplingConfig,
    ) -> Result<(u64, u64), AUTDInternalError> {
        if freq.hz() as f32 >= sampling_config.freq()?.hz() / 2. {
            return Err(AUTDInternalError::ModulationError(format!(
                "Frequency ({}) is equal to or greater than the Nyquist frequency ({})",
                freq,
                sampling_config.freq()? / 2.
            )));
        }
        if freq.hz() == 0 {
            return Err(AUTDInternalError::ModulationError(
                "Frequency must not be zero. If intentional, Use `Static` instead.".to_string(),
            ));
        }

        let fd = freq.hz() as u64 * sampling_config.division()? as u64;
        let fs = ULTRASOUND_FREQ.hz() as u64;

        let k = gcd(fs, fd);
        Ok((fs / k, fd / k))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExactFreqFloat;

impl SamplingMode for ExactFreqFloat {
    type T = Freq<f32>;
    fn validate(
        freq: Freq<f32>,
        sampling_config: SamplingConfig,
    ) -> Result<(u64, u64), AUTDInternalError> {
        if freq.hz() < 0. {
            return Err(AUTDInternalError::ModulationError(format!(
                "Frequency ({}) must be positive",
                freq
            )));
        }
        if freq.hz() == 0. {
            return Err(AUTDInternalError::ModulationError(
                "Frequency must not be zero. If intentional, Use `Static` instead.".to_string(),
            ));
        }
        if freq.hz() >= sampling_config.freq()?.hz() / 2. {
            return Err(AUTDInternalError::ModulationError(format!(
                "Frequency ({}) is equal to or greater than the Nyquist frequency ({})",
                freq,
                sampling_config.freq()? / 2.
            )));
        }
        let fd = freq.hz() as f64 * sampling_config.division()? as f64;
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
            "Frequency ({}) cannot be output with the sampling config ({}).",
            freq, sampling_config
        )))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NearestFreq;

impl SamplingMode for NearestFreq {
    type T = Freq<f32>;
    fn validate(
        freq: Freq<f32>,
        sampling_config: SamplingConfig,
    ) -> Result<(u64, u64), AUTDInternalError> {
        if freq.hz() < 0. {
            return Err(AUTDInternalError::ModulationError(format!(
                "Frequency ({}) must be positive",
                freq
            )));
        }
        if freq.hz() == 0. {
            return Err(AUTDInternalError::ModulationError(
                "Frequency must not be zero. If intentional, Use `Static` instead.".to_string(),
            ));
        }
        if freq.hz() >= sampling_config.freq()?.hz() / 2. {
            return Err(AUTDInternalError::ModulationError(format!(
                "Frequency ({}) is equal to or greater than the Nyquist frequency ({})",
                freq,
                sampling_config.freq()? / 2.
            )));
        }
        Ok(((sampling_config.freq()? / freq.hz()).hz().round() as u64, 1))
    }
}

pub trait SamplingModeInference: Copy + Clone + std::fmt::Debug + PartialEq {
    type T: SamplingMode<T = Self>;
}

impl SamplingModeInference for Freq<u32> {
    type T = ExactFreq;
}

impl SamplingModeInference for Freq<f32> {
    type T = ExactFreqFloat;
}
