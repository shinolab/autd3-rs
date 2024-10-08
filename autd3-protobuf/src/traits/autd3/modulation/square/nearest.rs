use autd3_driver::derive::ModulationProperty;

use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

impl ToMessage for autd3::modulation::Square<autd3::modulation::sampling_mode::NearestFreq> {
    type Message = Datagram;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram::Datagram::Modulation(Modulation {
                modulation: Some(modulation::Modulation::SquareNearest(SquareNearest {
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

impl FromMessage<SquareNearest>
    for autd3::modulation::Square<autd3::modulation::sampling_mode::NearestFreq>
{
    fn from_msg(msg: &SquareNearest) -> Result<Self, AUTDProtoBufError> {
        let mut square =
            autd3::modulation::Square::new_nearest(msg.freq * autd3_driver::defined::Hz);
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
    use autd3::modulation::sampling_mode::NearestFreq;
    use autd3_driver::defined::Hz;
    use rand::Rng;

    #[test]
    fn test_square() {
        let mut rng = rand::thread_rng();

        let m = autd3::modulation::Square::new_nearest(rng.gen::<f32>() * Hz)
            .with_high(rng.gen())
            .with_low(rng.gen())
            .with_duty(rng.gen());
        let msg = m.to_msg(None);

        match msg.datagram {
            Some(datagram::Datagram::Modulation(Modulation {
                modulation: Some(modulation::Modulation::SquareNearest(modulation)),
                ..
            })) => {
                let m2 = autd3::modulation::Square::<NearestFreq>::from_msg(&modulation).unwrap();
                approx::assert_abs_diff_eq!(m.freq().hz(), m2.freq().hz());
                assert_eq!(m.high(), m2.high());
                assert_eq!(m.low(), m2.low());
                approx::assert_abs_diff_eq!(m.duty(), m2.duty());
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
