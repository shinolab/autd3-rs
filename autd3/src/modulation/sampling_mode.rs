use autd3_driver::{
    derive::SamplingConfiguration, error::AUTDInternalError, firmware::fpga::ULTRASOUND_PERIOD,
    utils::float::is_integer,
};
use num::integer::gcd;

pub trait SamplingMode: Clone + Sync {
    fn validate(
        freq: f64,
        sampling_config: SamplingConfiguration,
        ultrasound_freq: u32,
    ) -> Result<(u64, u64), AUTDInternalError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExactFrequency;

impl SamplingMode for ExactFrequency {
    fn validate(
        freq: f64,
        sampling_config: SamplingConfiguration,
        ultrasound_freq: u32,
    ) -> Result<(u64, u64), AUTDInternalError> {
        let fd = freq * sampling_config.division(ultrasound_freq)? as f64;
        if !is_integer(fd) {
            return Err(AUTDInternalError::ModulationError(format!(
                "Frequency ({}Hz) cannot be output with the sampling config ({}).",
                freq, sampling_config
            )));
        }
        let fd = fd as u64;
        let fs = (ultrasound_freq * ULTRASOUND_PERIOD) as u64;

        let k = gcd(fs, fd);
        Ok((fs / k, fd / k))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NearestFrequency;

impl SamplingMode for NearestFrequency {
    fn validate(
        freq: f64,
        sampling_config: SamplingConfiguration,
        ultrasound_freq: u32,
    ) -> Result<(u64, u64), AUTDInternalError> {
        Ok((
            (sampling_config.freq(ultrasound_freq)? / freq).round() as u64,
            1,
        ))
    }
}
