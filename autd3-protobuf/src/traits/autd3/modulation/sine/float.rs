use autd3_driver::derive::ModulationProperty;

use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3::modulation::Sine<autd3::modulation::sampling_mode::NearestFreq> {
    type Message = DatagramLightweight;

    #[allow(clippy::unnecessary_cast)]
    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram_lightweight::Datagram::Modulation(Modulation {
                modulation: Some(modulation::Modulation::SineNearest(SineNearest {
                    config: Some(self.sampling_config().to_msg(None)),
                    freq: self.freq() as _,
                    intensity: self.intensity() as _,
                    offset: self.offset() as _,
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
        autd3::modulation::Sine<autd3::modulation::sampling_mode::NearestFreq>,
    >
{
    type Message = DatagramLightweight;

    #[allow(clippy::unnecessary_cast)]
    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram_lightweight::Datagram::Modulation(Modulation {
                modulation: Some(modulation::Modulation::SineNearest(SineNearest {
                    config: Some(self.sampling_config().to_msg(None)),
                    freq: self.freq() as _,
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

impl FromMessage<SineNearest>
    for autd3::modulation::Sine<autd3::modulation::sampling_mode::NearestFreq>
{
    #[allow(clippy::unnecessary_cast)]
    fn from_msg(msg: &SineNearest) -> Option<Self> {
        Some(
            autd3::modulation::Sine::with_freq_nearest(msg.freq as _)
                .with_intensity(msg.intensity as _)
                .with_offset(msg.offset as _)
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
    use autd3::modulation::sampling_mode::NearestFreq;
    use autd3_driver::firmware::fpga::Phase;
    use rand::Rng;

    #[test]
    fn test_sine() {
        let mut rng = rand::thread_rng();

        let m = autd3::modulation::Sine::with_freq_nearest(rng.gen())
            .with_intensity(rng.gen())
            .with_offset(rng.gen())
            .with_phase(Phase::new(rng.gen()));
        let msg = m.to_msg(None);

        match msg.datagram {
            Some(datagram_lightweight::Datagram::Modulation(Modulation {
                modulation: Some(modulation::Modulation::SineNearest(modulation)),
                ..
            })) => {
                let m2 = autd3::modulation::Sine::<NearestFreq>::from_msg(&modulation).unwrap();
                assert_approx_eq::assert_approx_eq!(m.freq(), m2.freq());
                assert_eq!(m.intensity(), m2.intensity());
                assert_eq!(m.offset(), m2.offset());
                assert_eq!(m.phase(), m2.phase());
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
