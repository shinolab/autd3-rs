use crate::{AUTDProtoBufError, FromMessage, SineOption};

mod exact;
mod exact_float;
mod nearest;

impl From<autd3::modulation::SineOption> for SineOption {
    fn from(value: autd3::modulation::SineOption) -> Self {
        Self {
            intensity: Some(value.intensity.into()),
            offset: Some(value.offset.into()),
            phase: Some(value.phase.into()),
            clamp: Some(value.clamp),
            config: Some(value.sampling_config.into()),
        }
    }
}

impl FromMessage<SineOption> for autd3::modulation::SineOption {
    fn from_msg(msg: SineOption) -> Result<Self, AUTDProtoBufError> {
        let default = autd3::modulation::SineOption::default();
        Ok(autd3::modulation::SineOption {
            intensity: msg
                .intensity
                .map(u8::try_from)
                .transpose()?
                .unwrap_or(default.intensity),
            offset: msg
                .offset
                .map(u8::try_from)
                .transpose()?
                .unwrap_or(default.offset),
            phase: msg
                .phase
                .map(autd3_core::common::Angle::from_msg)
                .transpose()?
                .unwrap_or(default.phase),
            clamp: msg.clamp.unwrap_or(false),
            sampling_config: msg
                .config
                .map(autd3_driver::firmware::fpga::SamplingConfig::from_msg)
                .transpose()?
                .unwrap_or(default.sampling_config),
        })
    }
}
