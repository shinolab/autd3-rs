use autd3_core::modulation::ModulationProperty;

use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

impl ToMessage for autd3::modulation::Square<autd3::modulation::sampling_mode::ExactFreqFloat> {
    type Message = Datagram;

    fn to_msg(&self, _: Option<&autd3_core::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram::Datagram::Modulation(Modulation {
                modulation: Some(modulation::Modulation::SquareExactFloat(SquareExactFloat {
                    config: Some(self.sampling_config().to_msg(None)),
                    freq: self.freq().hz() as _,
                    high: Some(self.high() as _),
                    low: Some(self.low() as _),
                    duty: Some(self.duty() as _),
                })),
                loop_behavior: Some(self.loop_behavior().to_msg(None)),
            })),
            timeout: None,
            parallel_threshold: None,
        }
    }
}

impl FromMessage<SquareExactFloat>
    for autd3::modulation::Square<autd3::modulation::sampling_mode::ExactFreqFloat>
{
    fn from_msg(msg: &SquareExactFloat) -> Result<Self, AUTDProtoBufError> {
        let mut square = autd3::modulation::Square::new(msg.freq * autd3_core::defined::Hz);
        if let Some(high) = msg.high {
            square = square.with_high(high as _);
        }
        if let Some(low) = msg.low {
            square = square.with_low(low as _);
        }
        if let Some(duty) = msg.duty {
            square = square.with_duty(duty as _);
        }
        if let Some(config) = msg.config.as_ref() {
            square = square.with_sampling_config(
                autd3_driver::firmware::fpga::SamplingConfig::from_msg(config)?,
            )?;
        }
        Ok(square)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3::modulation::sampling_mode::ExactFreqFloat;
    use autd3_core::defined::Hz;
    use rand::Rng;

    #[test]
    fn test_square() {
        let mut rng = rand::thread_rng();

        let m = autd3::modulation::Square::new(rng.gen::<f32>() * Hz)
            .with_high(rng.gen())
            .with_low(rng.gen())
            .with_duty(rng.gen());
        let msg = m.to_msg(None);

        match msg.datagram {
            Some(datagram::Datagram::Modulation(Modulation {
                modulation: Some(modulation::Modulation::SquareExactFloat(modulation)),
                ..
            })) => {
                let m2 =
                    autd3::modulation::Square::<ExactFreqFloat>::from_msg(&modulation).unwrap();
                assert_eq!(m.freq(), m2.freq());
                assert_eq!(m.high(), m2.high());
                assert_eq!(m.low(), m2.low());
                approx::assert_abs_diff_eq!(m.duty(), m2.duty());
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
