use crate::{AUTDProtoBufError, FromMessage, SquareOption, ToMessage};

mod exact;
mod exact_float;
mod nearest;

impl ToMessage for autd3::modulation::SquareOption {
    type Message = SquareOption;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(Self::Message {
            config: Some(self.sampling_config.to_msg(None)?),
            low: Some(self.low as _),
            high: Some(self.high as _),
            duty: Some(self.duty as _),
        })
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
