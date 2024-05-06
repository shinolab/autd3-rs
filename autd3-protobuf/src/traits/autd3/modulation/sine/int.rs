use autd3_driver::derive::ModulationProperty;

use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3::modulation::Sine<autd3::modulation::sampling_mode::ExactFrequency> {
    type Message = DatagramLightweight;

    #[allow(clippy::unnecessary_cast)]
    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram_lightweight::Datagram::Modulation(Modulation {
                modulation: Some(modulation::Modulation::SineExact(SineExact {
                    config: Some(self.sampling_config().to_msg(None)),
                    freq: self.freq() as _,
                    intensity: Some(self.intensity().to_msg(None)),
                    offset: Some(self.offset().to_msg(None)),
                    phase: Some(self.phase().to_msg(None)),
                })),
                segment: Segment::S0 as _,
                transition_mode: Some(TransitionMode::SyncIdx.into()),
                transition_value: Some(0),
            })),
        }
    }
}

impl ToMessage
    for autd3_driver::datagram::DatagramWithSegment<
        autd3::modulation::Sine<autd3::modulation::sampling_mode::ExactFrequency>,
    >
{
    type Message = DatagramLightweight;

    #[allow(clippy::unnecessary_cast)]
    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram_lightweight::Datagram::Modulation(Modulation {
                modulation: Some(modulation::Modulation::SineExact(SineExact {
                    config: Some(self.sampling_config().to_msg(None)),
                    freq: self.freq() as _,
                    intensity: Some(self.intensity().to_msg(None)),
                    offset: Some(self.offset().to_msg(None)),
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
    for autd3::modulation::Sine<autd3::modulation::sampling_mode::ExactFrequency>
{
    #[allow(clippy::unnecessary_cast)]
    fn from_msg(msg: &SineExact) -> Option<Self> {
        Some(
            Self::new(msg.freq as _)
                .with_intensity(autd3_driver::firmware::fpga::EmitIntensity::from_msg(
                    msg.intensity.as_ref()?,
                )?)
                .with_offset(autd3_driver::firmware::fpga::EmitIntensity::from_msg(
                    msg.offset.as_ref()?,
                )?)
                .with_phase(autd3_driver::firmware::fpga::Phase::from_msg(
                    msg.phase.as_ref()?,
                )?)
                .with_sampling_config(autd3_driver::firmware::fpga::SamplingConfig::from_msg(
                    msg.config.as_ref()?,
                )?),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3_driver::firmware::fpga::{EmitIntensity, Phase};
    use rand::Rng;

    #[test]
    fn test_sine() {
        let mut rng = rand::thread_rng();

        let m = autd3::modulation::Sine::<usize>::new(rng.gen())
            .with_intensity(EmitIntensity::new(rng.gen()))
            .with_offset(EmitIntensity::new(rng.gen()))
            .with_phase(Phase::new(rng.gen()));
        let msg = m.to_msg(None);

        match msg.datagram {
            Some(datagram_lightweight::Datagram::Modulation(Modulation {
                modulation: Some(modulation::Modulation::SineExact(modulation)),
                ..
            })) => {
                let m2 = autd3::modulation::Sine::<usize>::from_msg(&modulation).unwrap();
                assert_eq!(m.freq(), m2.freq());
                assert_eq!(m.intensity(), m2.intensity());
                assert_eq!(m.offset(), m2.offset());
                assert_eq!(m.phase(), m2.phase());
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
