use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3::gain::Uniform {
    type Message = DatagramLightweight;

    #[allow(clippy::unnecessary_cast)]
    fn to_msg(&self) -> Self::Message {
        Self::Message {
            datagram: Some(datagram_lightweight::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Uniform(Uniform {
                    intensity: Some(self.intensity().to_msg()),
                    phase: Some(self.phase().to_msg()),
                })),
            })),
        }
    }
}

impl FromMessage<Uniform> for autd3::gain::Uniform {
    #[allow(clippy::unnecessary_cast)]
    fn from_msg(msg: &Uniform) -> Option<Self> {
        Some(
            Self::new(autd3_driver::common::EmitIntensity::from_msg(
                msg.intensity.as_ref()?,
            )?)
            .with_phase(autd3_driver::common::Phase::from_msg(msg.phase.as_ref()?)?),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3_driver::common::{EmitIntensity, Phase};
    use rand::Rng;

    #[test]
    fn test_phase() {
        let mut rng = rand::thread_rng();

        let g = autd3::gain::Uniform::new(EmitIntensity::new(rng.gen()))
            .with_phase(Phase::new(rng.gen()));
        let msg = g.to_msg();

        match msg.datagram {
            Some(datagram_lightweight::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Uniform(gain)),
            })) => {
                let g2 = autd3::gain::Uniform::from_msg(&gain).unwrap();
                assert_eq!(g.intensity(), g2.intensity());
                assert_eq!(g.phase(), g2.phase());
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}