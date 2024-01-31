use autd3_driver::derive::ModulationProperty;

use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3::modulation::Sine {
    type Message = DatagramLightweight;

    #[allow(clippy::unnecessary_cast)]
    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram_lightweight::Datagram::Modulation(Modulation {
                modulation: Some(modulation::Modulation::Sine(Sine {
                    config: Some(self.sampling_config().to_msg(None)),
                    freq: self.freq() as _,
                    intensity: Some(self.intensity().to_msg(None)),
                    offset: Some(self.offset().to_msg(None)),
                    phase: Some(self.phase().to_msg(None)),
                    mode: SamplingMode::from(self.mode()).into(),
                })),
            })),
        }
    }
}

impl FromMessage<Sine> for autd3::modulation::Sine {
    #[allow(clippy::unnecessary_cast)]
    fn from_msg(msg: &Sine) -> Option<Self> {
        Some(
            Self::new(msg.freq as _)
                .with_intensity(autd3_driver::common::EmitIntensity::from_msg(
                    msg.intensity.as_ref()?,
                )?)
                .with_offset(autd3_driver::common::EmitIntensity::from_msg(
                    msg.offset.as_ref()?,
                )?)
                .with_phase(autd3_driver::common::Phase::from_msg(msg.phase.as_ref()?)?)
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
    use autd3_driver::common::{EmitIntensity, Phase};
    use rand::Rng;

    #[test]
    fn test_sine() {
        let mut rng = rand::thread_rng();

        let m = autd3::modulation::Sine::new(rng.gen())
            .with_intensity(EmitIntensity::new(rng.gen()))
            .with_offset(EmitIntensity::new(rng.gen()))
            .with_phase(Phase::new(rng.gen()))
            .with_mode(autd3::modulation::SamplingMode::SizeOptimized);
        let msg = m.to_msg(None);

        match msg.datagram {
            Some(datagram_lightweight::Datagram::Modulation(Modulation {
                modulation: Some(modulation::Modulation::Sine(modulation)),
            })) => {
                let m2 = autd3::modulation::Sine::from_msg(&modulation).unwrap();
                assert_approx_eq::assert_approx_eq!(m.freq(), m2.freq());
                assert_eq!(m.intensity(), m2.intensity());
                assert_eq!(m.offset(), m2.offset());
                assert_eq!(m.phase(), m2.phase());
                assert_eq!(m.mode(), m2.mode());
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
