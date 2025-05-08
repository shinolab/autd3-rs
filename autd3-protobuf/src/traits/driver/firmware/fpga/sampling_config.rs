use autd3::driver::defined::Hz;

use crate::{AUTDProtoBufError, pb::*, traits::FromMessage};
use std::{num::NonZeroU16, time::Duration};

impl From<autd3_driver::firmware::fpga::SamplingConfig> for SamplingConfig {
    fn from(value: autd3_driver::firmware::fpga::SamplingConfig) -> Self {
        match value {
            autd3_driver::firmware::fpga::SamplingConfig::Divide(div) => SamplingConfig {
                variant: Some(sampling_config::Variant::Divide(sampling_config::Divide {
                    div: div.get() as _,
                })),
            },
            autd3_driver::firmware::fpga::SamplingConfig::Freq(freq) => SamplingConfig {
                variant: Some(sampling_config::Variant::Freq(sampling_config::Freq {
                    freq: freq.hz(),
                })),
            },
            autd3_driver::firmware::fpga::SamplingConfig::FreqNearest(freq) => SamplingConfig {
                variant: Some(sampling_config::Variant::FreqNearest(
                    sampling_config::FreqNearest { freq: freq.0.hz() },
                )),
            },
            autd3_driver::firmware::fpga::SamplingConfig::Period(period) => SamplingConfig {
                variant: Some(sampling_config::Variant::Period(sampling_config::Period {
                    ns: period.as_nanos() as _,
                })),
            },
            autd3_driver::firmware::fpga::SamplingConfig::PeriodNearest(period) => SamplingConfig {
                variant: Some(sampling_config::Variant::PeriodNearest(
                    sampling_config::PeriodNearest {
                        ns: period.0.as_nanos() as _,
                    },
                )),
            },
        }
    }
}

impl FromMessage<SamplingConfig> for autd3_driver::firmware::fpga::SamplingConfig {
    fn from_msg(msg: SamplingConfig) -> Result<Self, AUTDProtoBufError> {
        Ok(
            match msg.variant.ok_or(AUTDProtoBufError::DataParseError)? {
                sampling_config::Variant::Divide(divide) => {
                    autd3_driver::firmware::fpga::SamplingConfig::Divide(NonZeroU16::try_from(
                        u16::try_from(divide.div)?,
                    )?)
                }
                sampling_config::Variant::Freq(freq) => {
                    autd3_driver::firmware::fpga::SamplingConfig::Freq(freq.freq * Hz)
                }
                sampling_config::Variant::FreqNearest(freq) => {
                    autd3_driver::firmware::fpga::SamplingConfig::Freq(freq.freq * Hz)
                        .into_nearest()
                }
                sampling_config::Variant::Period(period) => {
                    autd3_driver::firmware::fpga::SamplingConfig::Period(Duration::from_nanos(
                        period.ns,
                    ))
                }
                sampling_config::Variant::PeriodNearest(period) => {
                    autd3_driver::firmware::fpga::SamplingConfig::Period(Duration::from_nanos(
                        period.ns,
                    ))
                    .into_nearest()
                }
            },
        )
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
        let msg = v.into();
        let v2 = SamplingConfig::from_msg(msg).unwrap();
        assert_eq!(v, v2);
    }
}
