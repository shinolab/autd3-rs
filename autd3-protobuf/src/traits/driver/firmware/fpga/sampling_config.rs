use std::num::NonZeroU16;
use std::time::Duration;

use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

impl ToMessage for autd3_driver::firmware::fpga::SamplingConfig {
    type Message = SamplingConfig;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            config: Some(match *self {
                autd3::derive::SamplingConfig::Freq(value) => {
                    sampling_config::Config::Freq(SamplingConfigFreq { value: value.hz() })
                }
                autd3::derive::SamplingConfig::FreqNearest(value) => {
                    sampling_config::Config::FreqNearest(SamplingConfigFreqNearest {
                        value: value.hz(),
                    })
                }
                autd3::derive::SamplingConfig::Period(value) => {
                    sampling_config::Config::Period(SamplingConfigPeriod {
                        value: value.as_nanos() as _,
                    })
                }
                autd3::derive::SamplingConfig::PeriodNearest(value) => {
                    sampling_config::Config::PeriodNearest(SamplingConfigPeriodNearest {
                        value: value.as_nanos() as _,
                    })
                }
                autd3::derive::SamplingConfig::Division(value) => {
                    sampling_config::Config::Division(SamplingConfigDivision {
                        value: value.get() as _,
                    })
                }
                _ => unimplemented!(),
            }),
        }
    }
}

impl FromMessage<SamplingConfig> for autd3_driver::firmware::fpga::SamplingConfig {
    fn from_msg(msg: &SamplingConfig) -> Result<Self, AUTDProtoBufError> {
        msg.config
            .as_ref()
            .ok_or(AUTDProtoBufError::DataParseError)
            .and_then(|config| match *config {
                sampling_config::Config::Freq(SamplingConfigFreq { value }) => {
                    Ok(autd3_driver::firmware::fpga::SamplingConfig::Freq(
                        value * autd3_driver::defined::Hz,
                    ))
                }
                sampling_config::Config::FreqNearest(SamplingConfigFreqNearest { value }) => {
                    Ok(autd3_driver::firmware::fpga::SamplingConfig::FreqNearest(
                        value * autd3_driver::defined::Hz,
                    ))
                }
                sampling_config::Config::Period(SamplingConfigPeriod { value }) => {
                    Ok(autd3_driver::firmware::fpga::SamplingConfig::Period(
                        Duration::from_nanos(value),
                    ))
                }
                sampling_config::Config::PeriodNearest(SamplingConfigPeriodNearest { value }) => {
                    Ok(autd3_driver::firmware::fpga::SamplingConfig::PeriodNearest(
                        Duration::from_nanos(value),
                    ))
                }
                sampling_config::Config::Division(SamplingConfigDivision { value }) => {
                    Ok(autd3_driver::firmware::fpga::SamplingConfig::Division(
                        NonZeroU16::new(value as _).ok_or(AUTDProtoBufError::DataParseError)?,
                    ))
                }
            })
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
        let v = SamplingConfig::Division(NonZeroU16::new(rng.gen_range(0x0001..=0xFFFF)).unwrap());
        let msg = v.to_msg(None);
        let v2 = SamplingConfig::from_msg(&msg).unwrap();
        assert_eq!(v, v2);
    }
}
