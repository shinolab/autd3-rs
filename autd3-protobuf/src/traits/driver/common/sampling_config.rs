use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3_driver::common::SamplingConfiguration {
    type Message = SamplingConfiguration;

    fn to_msg(&self) -> Self::Message {
        Self::Message {
            freq_div: self.frequency_division(),
        }
    }
}

impl FromMessage<SamplingConfiguration> for autd3_driver::common::SamplingConfiguration {
    fn from_msg(msg: &SamplingConfiguration) -> Option<Self> {
        autd3_driver::common::SamplingConfiguration::from_frequency_division(msg.freq_div).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3_driver::{
        derive::SamplingConfiguration,
        fpga::{SAMPLING_FREQ_DIV_MAX, SAMPLING_FREQ_DIV_MIN},
    };
    use rand::Rng;

    #[test]
    fn test_sampling_config() {
        let mut rng = rand::thread_rng();
        let v = SamplingConfiguration::from_frequency_division(
            rng.gen_range(SAMPLING_FREQ_DIV_MIN..SAMPLING_FREQ_DIV_MAX),
        )
        .unwrap();
        let msg = v.to_msg();
        let v2 = SamplingConfiguration::from_msg(&msg).unwrap();
        assert_eq!(v, v2);
    }
}
