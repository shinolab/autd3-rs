use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

impl ToMessage for autd3::gain::Uniform {
    type Message = Datagram;

    fn to_msg(&self, _: Option<&autd3_core::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Uniform(Uniform {
                    intensity: Some(self.drive().intensity().to_msg(None)),
                    phase: Some(self.drive().phase().to_msg(None)),
                })),
            })),
            timeout: None,
            parallel_threshold: None,
        }
    }
}

impl FromMessage<Uniform> for autd3::gain::Uniform {
    fn from_msg(msg: &Uniform) -> Result<Self, AUTDProtoBufError> {
        Ok(Self::new((
            autd3_driver::firmware::fpga::Phase::from_msg(
                msg.phase
                    .as_ref()
                    .ok_or(AUTDProtoBufError::DataParseError)?,
            )?,
            autd3_driver::firmware::fpga::EmitIntensity::from_msg(
                msg.intensity
                    .as_ref()
                    .ok_or(AUTDProtoBufError::DataParseError)?,
            )?,
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3_driver::firmware::fpga::{EmitIntensity, Phase};
    use rand::Rng;

    #[test]
    fn test_phase() {
        let mut rng = rand::thread_rng();

        let g = autd3::gain::Uniform::new((EmitIntensity::new(rng.gen()), Phase::new(rng.gen())));
        let msg = g.to_msg(None);

        match msg.datagram {
            Some(datagram::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Uniform(gain)),
                ..
            })) => {
                let g2 = autd3::gain::Uniform::from_msg(&gain).unwrap();
                assert_eq!(g.drive().intensity(), g2.drive().intensity());
                assert_eq!(g.drive().phase(), g2.drive().phase());
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
