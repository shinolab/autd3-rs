use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3_driver::firmware::fpga::SamplingConfig {
    type Message = SamplingConfig;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            config: Some(match *self {
                autd3::derive::SamplingConfig::Freq(value) => {
                    sampling_config::Config::Freq(SamplingConfigFreq { value })
                }
                autd3::derive::SamplingConfig::FreqNearest(value) => {
                    sampling_config::Config::FreqNearest(SamplingConfigFreqNearest {
                        value: value as f32,
                    })
                }
                autd3::derive::SamplingConfig::DivisionRaw(value) => {
                    sampling_config::Config::DivisionRaw(SamplingConfigDivisionRaw { value })
                }
                autd3::derive::SamplingConfig::Division(value) => {
                    sampling_config::Config::Division(SamplingConfigDivision { value })
                }
            }),
        }
    }
}

impl FromMessage<SamplingConfig> for autd3_driver::firmware::fpga::SamplingConfig {
    fn from_msg(msg: &SamplingConfig) -> Option<Self> {
        msg.config.as_ref().map(|config| match *config {
            sampling_config::Config::Freq(SamplingConfigFreq { value }) => {
                autd3_driver::firmware::fpga::SamplingConfig::Freq(value)
            }
            sampling_config::Config::FreqNearest(SamplingConfigFreqNearest { value }) => {
                autd3_driver::firmware::fpga::SamplingConfig::FreqNearest(value as f64)
            }
            sampling_config::Config::Division(SamplingConfigDivision { value }) => {
                autd3_driver::firmware::fpga::SamplingConfig::Division(value)
            }
            sampling_config::Config::DivisionRaw(SamplingConfigDivisionRaw { value }) => {
                autd3_driver::firmware::fpga::SamplingConfig::DivisionRaw(value)
            }
        })
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
