use crate::{AUTDProtoBufError, FromMessage, SineOption, ToMessage};

mod exact;
mod exact_float;
mod nearest;

impl ToMessage for autd3::modulation::SineOption {
    type Message = SineOption;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(Self::Message {
            config: Some(self.sampling_config.to_msg(None)?),
            intensity: Some(self.intensity as _),
            offset: Some(self.offset as _),
            phase: Some(self.phase.to_msg(None)?),
            clamp: Some(self.clamp),
        })
    }
}

impl FromMessage<SineOption> for autd3::modulation::SineOption {
    fn from_msg(msg: &SineOption) -> Result<Self, AUTDProtoBufError> {
        Ok(autd3::modulation::SineOption {
            intensity: msg
                .intensity
                .map(u8::try_from)
                .transpose()?
                .unwrap_or(autd3::modulation::SineOption::default().intensity),
            offset: msg
                .offset
                .map(u8::try_from)
                .transpose()?
                .unwrap_or(autd3::modulation::SineOption::default().offset),
            phase: autd3_core::defined::Angle::from_msg(&msg.phase)?,
            clamp: msg.clamp.unwrap_or(false),
            sampling_config: msg
                .config
                .as_ref()
                .map(autd3_driver::firmware::fpga::SamplingConfig::from_msg)
                .transpose()?
                .unwrap_or(autd3::modulation::SineOption::default().sampling_config),
        })
    }
}
