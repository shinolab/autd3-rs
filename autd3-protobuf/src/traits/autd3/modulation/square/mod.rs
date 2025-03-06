use crate::{AUTDProtoBufError, FromMessage, SquareOption};

mod exact;
mod exact_float;
mod nearest;

impl From<autd3::modulation::SquareOption> for SquareOption {
    fn from(value: autd3::modulation::SquareOption) -> Self {
        Self {
            low: Some(value.low.into()),
            high: Some(value.high.into()),
            duty: Some(value.duty),
            config: Some(value.sampling_config.into()),
        }
    }
}

impl FromMessage<SquareOption> for autd3::modulation::SquareOption {
    fn from_msg(msg: SquareOption) -> Result<Self, AUTDProtoBufError> {
        let default = autd3::modulation::SquareOption::default();
        Ok(autd3::modulation::SquareOption {
            low: msg
                .low
                .map(u8::try_from)
                .transpose()?
                .unwrap_or(default.low),
            high: msg
                .high
                .map(u8::try_from)
                .transpose()?
                .unwrap_or(default.high),
            duty: msg.duty.unwrap_or(default.duty),
            sampling_config: msg
                .config
                .map(autd3_driver::firmware::fpga::SamplingConfig::from_msg)
                .transpose()?
                .unwrap_or(default.sampling_config),
        })
    }
}
