use autd3_driver::derive::ModulationProperty;

use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3::modulation::Square {
    type Message = DatagramLightweight;

    #[allow(clippy::unnecessary_cast)]
    fn to_msg(&self) -> Self::Message {
        Self::Message {
            datagram: Some(datagram_lightweight::Datagram::Modulation(Modulation {
                modulation: Some(modulation::Modulation::Square(Square {
                    config: Some(self.sampling_config().to_msg()),
                    freq: self.freq() as _,
                    high: Some(self.high().to_msg()),
                    low: Some(self.low().to_msg()),
                    duty: self.duty() as _,
                    mode: SamplingMode::from(self.mode()).into(),
                })),
            })),
        }
    }
}

impl FromMessage<Square> for autd3::modulation::Square {
    #[allow(clippy::unnecessary_cast)]
    fn from_msg(msg: &Square) -> Option<Self> {
        Some(
            Self::new(msg.freq as _)
                .with_high(autd3_driver::common::EmitIntensity::from_msg(
                    msg.high.as_ref()?,
                )?)
                .with_low(autd3_driver::common::EmitIntensity::from_msg(
                    msg.low.as_ref()?,
                )?)
                .with_duty(msg.duty as _)
                .with_sampling_config(autd3_driver::common::SamplingConfiguration::from_msg(
                    msg.config.as_ref()?,
                )?)
                .with_mode(autd3::modulation::SamplingMode::from(
                    SamplingMode::try_from(msg.mode).ok()?,
                )),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3_driver::common::EmitIntensity;
    use rand::Rng;

    #[test]
    fn test_sine() {
        let mut rng = rand::thread_rng();

        let m = autd3::modulation::Square::new(rng.gen())
            .with_high(EmitIntensity::new(rng.gen()))
            .with_low(EmitIntensity::new(rng.gen()))
            .with_duty(rng.gen())
            .with_mode(autd3::modulation::SamplingMode::SizeOptimized);
        let msg = m.to_msg();

        match msg.datagram {
            Some(datagram_lightweight::Datagram::Modulation(Modulation {
                modulation: Some(modulation::Modulation::Square(modulation)),
            })) => {
                let m2 = autd3::modulation::Square::from_msg(&modulation).unwrap();
                assert_approx_eq::assert_approx_eq!(m.freq(), m2.freq());
                assert_eq!(m.high(), m2.high());
                assert_eq!(m.low(), m2.low());
                assert_approx_eq::assert_approx_eq!(m.duty(), m2.duty());
                assert_eq!(m.mode(), m2.mode());
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}