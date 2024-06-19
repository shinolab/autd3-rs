use autd3_driver::derive::ModulationProperty;

use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3::modulation::Sine<autd3::modulation::sampling_mode::ExactFreq> {
    type Message = DatagramLightweight;

    #[allow(clippy::unnecessary_cast)]
    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram_lightweight::Datagram::Modulation(Modulation {
                modulation: Some(modulation::Modulation::SineExact(SineExact {
                    config: Some(self.sampling_config().to_msg(None)),
                    freq: self.freq().hz() as _,
                    intensity: self.intensity() as _,
                    offset: self.offset() as _,
                    phase: Some(self.phase().to_msg(None)),
                })),
                segment: Segment::S0 as _,
                transition_mode: Some(TransitionMode::Immediate.into()),
                transition_value: Some(0),
            })),
        }
    }
}

impl ToMessage
    for autd3_driver::datagram::DatagramWithSegmentTransition<
        autd3::modulation::Sine<autd3::modulation::sampling_mode::ExactFreq>,
    >
{
    type Message = DatagramLightweight;

    #[allow(clippy::unnecessary_cast)]
    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram_lightweight::Datagram::Modulation(Modulation {
                modulation: Some(modulation::Modulation::SineExact(SineExact {
                    config: Some(self.sampling_config().to_msg(None)),
                    freq: self.freq().hz() as _,
                    intensity: self.intensity() as _,
                    offset: self.offset() as _,
                    phase: Some(self.phase().to_msg(None)),
                })),
                segment: self.segment() as _,
                transition_mode: self.transition_mode().map(|m| m.mode() as _),
                transition_value: self.transition_mode().map(|m| m.value()),
            })),
        }
    }
}

impl FromMessage<SineExact>
    for autd3::modulation::Sine<autd3::modulation::sampling_mode::ExactFreq>
{
    #[allow(clippy::unnecessary_cast)]
    fn from_msg(msg: &SineExact) -> Option<Self> {
        Some(
            autd3::modulation::Sine::new(msg.freq * autd3_driver::defined::Hz)
                .with_intensity(msg.intensity as u8)
                .with_offset(msg.offset as u8)
                .with_phase(autd3_driver::defined::Angle::from_msg(msg.phase.as_ref()?)?)
                .with_sampling_config(autd3_driver::firmware::fpga::SamplingConfig::from_msg(
                    msg.config.as_ref()?,
                )?),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3::modulation::sampling_mode::ExactFreq;
    use autd3_driver::defined::{rad, Hz};
    use rand::Rng;

    #[test]
    fn test_sine() {
        let mut rng = rand::thread_rng();

        let m = autd3::modulation::Sine::new(rng.gen::<u32>() * Hz)
            .with_intensity(rng.gen())
            .with_offset(rng.gen())
            .with_phase(rng.gen::<f32>() * rad);
        let msg = m.to_msg(None);

        match msg.datagram {
            Some(datagram_lightweight::Datagram::Modulation(Modulation {
                modulation: Some(modulation::Modulation::SineExact(modulation)),
                ..
            })) => {
                let m2 = autd3::modulation::Sine::<ExactFreq>::from_msg(&modulation).unwrap();
                assert_eq!(m.freq().hz(), m2.freq().hz());
                assert_eq!(m.intensity(), m2.intensity());
                assert_eq!(m.offset(), m2.offset());
                assert_eq!(m.phase(), m2.phase());
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
