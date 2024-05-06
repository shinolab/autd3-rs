use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3_driver::firmware::fpga::SamplingConfig {
    type Message = SamplingConfig;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            freq_div: self.division(),
        }
    }
}

impl FromMessage<SamplingConfig> for autd3_driver::firmware::fpga::SamplingConfig {
    fn from_msg(msg: &SamplingConfig) -> Option<Self> {
        autd3_driver::firmware::fpga::SamplingConfig::from_division_raw(msg.freq_div).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3_driver::{
        derive::SamplingConfig,
        firmware::fpga::{SAMPLING_FREQ_DIV_MAX, SAMPLING_FREQ_DIV_MIN},
    };
    use rand::Rng;

    #[test]
    fn test_sampling_config() {
        let mut rng = rand::thread_rng();
        let v = SamplingConfig::from_division_raw(
            rng.gen_range(SAMPLING_FREQ_DIV_MIN..SAMPLING_FREQ_DIV_MAX),
        )
        .unwrap();
        let msg = v.to_msg(None);
        let v2 = SamplingConfig::from_msg(&msg).unwrap();
        assert_eq!(v, v2);
    }
}
