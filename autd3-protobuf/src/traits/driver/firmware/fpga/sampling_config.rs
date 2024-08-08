use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

impl ToMessage for autd3_driver::firmware::fpga::SamplingConfig {
    type Message = SamplingConfig;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            div: self.division() as _,
        }
    }
}

impl FromMessage<SamplingConfig> for autd3_driver::firmware::fpga::SamplingConfig {
    fn from_msg(msg: &SamplingConfig) -> Result<Self, AUTDProtoBufError> {
        Self::new(msg.div as u16).map_err(|_| AUTDProtoBufError::DataParseError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3_driver::derive::SamplingConfig;
    use rand::Rng;

    #[test]
    fn test_sampling_config() {
        let mut rng = rand::thread_rng();
        let v = SamplingConfig::new(rng.gen_range(0x0001..=0xFFFF)).unwrap();
        let msg = v.to_msg(None);
        let v2 = SamplingConfig::from_msg(&msg).unwrap();
        assert_eq!(v, v2);
    }
}
