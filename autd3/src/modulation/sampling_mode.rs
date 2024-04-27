use autd3_driver::{
    derive::{EmitIntensity, SamplingConfiguration},
    error::AUTDInternalError,
    firmware::fpga::sampling_config,
    utils::float::is_integer,
};
use num::integer::gcd;

pub trait SamplingMode: Clone {
    type D;
    fn validate(
        freq: f64,
        sampling_config: SamplingConfiguration,
    ) -> Result<(u64, u64), AUTDInternalError> {
        let fd = freq * sampling_config.division() as f64;
        if !is_integer(fd) {
            return Err(AUTDInternalError::ModulationError(format!(
                "Frequency ({}Hz) cannot be output with the sampling config ({}).",
                freq, sampling_config
            )));
        }
        let fd = fd as u64;
        let fs = sampling_config::base_frequency() as u64;

        let k = gcd(fs, fd);
        Ok((fs / k, fd / k))
    }
    fn calc(
        freq: f64,
        sampling_config: SamplingConfiguration,
        data: Self::D,
    ) -> Result<Vec<EmitIntensity>, AUTDInternalError>;
}
