use autd3_driver::{
    defined::{Freq, Frequency},
    derive::SamplingConfig,
    error::AUTDInternalError,
    firmware::fpga::ULTRASOUND_PERIOD,
    utils::float::is_integer,
};
use num::integer::gcd;

pub trait SamplingMode: Clone + Sync {
    type T: Frequency;
    fn validate(
        freq: Self::T,
        sampling_config: SamplingConfig,
        ultrasound_freq: Freq<u32>,
    ) -> Result<(u64, u64), AUTDInternalError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExactFreq;

impl SamplingMode for ExactFreq {
    type T = Freq<u32>;
    fn validate(
        freq: Freq<u32>,
        sampling_config: SamplingConfig,
        ultrasound_freq: Freq<u32>,
    ) -> Result<(u64, u64), AUTDInternalError> {
        if freq.hz() as f64 >= sampling_config.freq(ultrasound_freq)?.hz() / 2. {
            return Err(AUTDInternalError::ModulationError(format!(
                "Frequency ({}) is equal to or greater than the Nyquist frequency ({})",
                freq,
                sampling_config.freq(ultrasound_freq)? / 2.
            )));
        }
        let fd = freq.hz() * sampling_config.division(ultrasound_freq)?;
        let fd = fd as u64;
        let fs = (ultrasound_freq * ULTRASOUND_PERIOD).hz() as u64;

        let k = gcd(fs, fd);
        Ok((fs / k, fd / k))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExactFreqFloat;

impl SamplingMode for ExactFreqFloat {
    type T = Freq<f64>;
    fn validate(
        freq: Freq<f64>,
        sampling_config: SamplingConfig,
        ultrasound_freq: Freq<u32>,
    ) -> Result<(u64, u64), AUTDInternalError> {
        if freq.hz() < 0. {
            return Err(AUTDInternalError::ModulationError(format!(
                "Frequency ({}) must be positive",
                freq
            )));
        }
        if freq.hz() >= sampling_config.freq(ultrasound_freq)?.hz() / 2. {
            return Err(AUTDInternalError::ModulationError(format!(
                "Frequency ({}) is equal to or greater than the Nyquist frequency ({})",
                freq,
                sampling_config.freq(ultrasound_freq)? / 2.
            )));
        }
        let fd = freq.hz() * sampling_config.division(ultrasound_freq)? as f64;
        if !is_integer(fd) {
            return Err(AUTDInternalError::ModulationError(format!(
                "Frequency ({}) cannot be output with the sampling config ({}).",
                freq, sampling_config
            )));
        }
        let fd = fd as u64;
        let fs = (ultrasound_freq * ULTRASOUND_PERIOD).hz() as u64;

        let k = gcd(fs, fd);
        Ok((fs / k, fd / k))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NearestFreq;

impl SamplingMode for NearestFreq {
    type T = Freq<f64>;
    fn validate(
        freq: Freq<f64>,
        sampling_config: SamplingConfig,
        ultrasound_freq: Freq<u32>,
    ) -> Result<(u64, u64), AUTDInternalError> {
        if freq.hz() < 0. {
            return Err(AUTDInternalError::ModulationError(format!(
                "Frequency ({}) must be positive",
                freq
            )));
        }
        if freq.hz() >= sampling_config.freq(ultrasound_freq)?.hz() / 2. {
            return Err(AUTDInternalError::ModulationError(format!(
                "Frequency ({}) is equal to or greater than the Nyquist frequency ({})",
                freq,
                sampling_config.freq(ultrasound_freq)? / 2.
            )));
        }
        Ok((
            (sampling_config.freq(ultrasound_freq)? / freq.hz())
                .hz()
                .round() as u64,
            1,
        ))
    }
}

pub trait SamplingModeInference: Copy + Clone + std::fmt::Debug + PartialEq {
    type T: SamplingMode<T = Self>;
}

impl SamplingModeInference for Freq<u32> {
    type T = ExactFreq;
}

impl SamplingModeInference for Freq<f64> {
    type T = ExactFreqFloat;
}
