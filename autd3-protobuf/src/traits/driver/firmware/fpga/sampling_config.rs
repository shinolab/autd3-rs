use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};
use std::num::NonZeroU16;

impl ToMessage for autd3_driver::firmware::fpga::SamplingConfig {
    type Message = SamplingConfig;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(Self::Message {
            div: self
                .division()
                .map_err(autd3_driver::error::AUTDDriverError::from)? as _,
        })
    }
}

impl FromMessage<SamplingConfig> for autd3_driver::firmware::fpga::SamplingConfig {
    fn from_msg(msg: &SamplingConfig) -> Result<Self, AUTDProtoBufError> {
        Ok(Self::new(
            NonZeroU16::new(u16::try_from(msg.div).map_err(|_| AUTDProtoBufError::DataParseError)?)
                .ok_or(AUTDProtoBufError::DataParseError)?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3_driver::firmware::fpga::SamplingConfig;
    use rand::Rng;

    #[test]
    fn test_sampling_config() {
        let mut rng = rand::rng();
        let v = SamplingConfig::new(NonZeroU16::new(rng.random_range(0x0001..=0xFFFF)).unwrap());
        let msg = v.to_msg(None).unwrap();
        let v2 = SamplingConfig::from_msg(&msg).unwrap();
        assert_eq!(v, v2);
    }
}
