use autd3_driver::derive::ModulationProperty;

use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3::modulation::Square<autd3::modulation::sampling_mode::NearestFreq> {
    type Message = DatagramLightweight;

    #[allow(clippy::unnecessary_cast)]
    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram_lightweight::Datagram::Modulation(Modulation {
                modulation: Some(modulation::Modulation::SquareNearest(SquareNearest {
                    config: Some(self.sampling_config().to_msg(None)),
                    freq: self.freq().hz() as _,
                    high: self.high().value() as _,
                    low: self.low().value() as _,
                    duty: self.duty() as _,
                })),
                segment: Segment::S0 as _,
                transition_mode: Some(TransitionMode::SyncIdx.into()),
                transition_value: Some(0),
            })),
        }
    }
}

impl ToMessage
    for autd3_driver::datagram::DatagramWithSegmentTransition<
        autd3::modulation::Square<autd3::modulation::sampling_mode::NearestFreq>,
    >
{
    type Message = DatagramLightweight;

    #[allow(clippy::unnecessary_cast)]
    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram_lightweight::Datagram::Modulation(Modulation {
                modulation: Some(modulation::Modulation::SquareNearest(SquareNearest {
                    config: Some(self.sampling_config().to_msg(None)),
                    freq: self.freq().hz() as _,
                    high: self.high().value() as _,
                    low: self.low().value() as _,
                    duty: self.duty() as _,
                })),
                segment: self.segment() as _,
                transition_mode: self.transition_mode().map(|m| m.mode() as _),
                transition_value: self.transition_mode().map(|m| m.value()),
            })),
        }
    }
}

impl FromMessage<SquareNearest>
    for autd3::modulation::Square<autd3::modulation::sampling_mode::NearestFreq>
{
    #[allow(clippy::unnecessary_cast)]
    fn from_msg(msg: &SquareNearest) -> Option<Self> {
        Some(
            autd3::modulation::Square::with_freq_nearest(
                (msg.freq as f64) * autd3_driver::defined::Hz,
            )
            .with_high(msg.high as u8)
            .with_low(msg.low as u8)
            .with_duty(msg.duty as _)
            .with_sampling_config(
                autd3_driver::firmware::fpga::SamplingConfig::from_msg(msg.config.as_ref()?)?,
            ),
        )
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

        let m = autd3::modulation::Square::with_freq_nearest(rng.gen::<f64>() * Hz)
            .with_high(rng.gen())
            .with_low(rng.gen())
            .with_duty(rng.gen());
        let msg = m.to_msg(None);

        match msg.datagram {
            Some(datagram_lightweight::Datagram::Modulation(Modulation {
                modulation: Some(modulation::Modulation::SquareNearest(modulation)),
                ..
            })) => {
                let m2 = autd3::modulation::Square::<NearestFreq>::from_msg(&modulation).unwrap();
                assert_approx_eq::assert_approx_eq!(m.freq().hz(), m2.freq().hz());
                assert_eq!(m.high(), m2.high());
                assert_eq!(m.low(), m2.low());
                assert_approx_eq::assert_approx_eq!(m.duty(), m2.duty());
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
