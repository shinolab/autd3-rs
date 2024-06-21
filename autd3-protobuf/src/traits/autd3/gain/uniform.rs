use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

impl ToMessage for autd3::gain::Uniform {
    type Message = Datagram;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Uniform(Uniform {
                    intensity: Some(self.intensity().to_msg(None)),
                    phase: Some(self.phase().to_msg(None)),
                })),
            })),
            timeout: None,
            parallel_threshold: None,
        }
    }
}

impl FromMessage<Uniform> for autd3::gain::Uniform {
    fn from_msg(msg: &Uniform) -> Result<Self, AUTDProtoBufError> {
        let mut g = Self::new(autd3_driver::firmware::fpga::EmitIntensity::from_msg(
            msg.intensity
                .as_ref()
                .ok_or(AUTDProtoBufError::DataParseError)?,
        )?);
        if let Some(phase) = msg.phase.as_ref() {
            g = g.with_phase(autd3_driver::firmware::fpga::Phase::from_msg(phase)?);
        }
        Ok(g)
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

        let g = autd3::gain::Uniform::new(EmitIntensity::new(rng.gen()))
            .with_phase(Phase::new(rng.gen()));
        let msg = g.to_msg(None);

        match msg.datagram {
            Some(datagram::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Uniform(gain)),
                ..
            })) => {
                let g2 = autd3::gain::Uniform::from_msg(&gain).unwrap();
                assert_eq!(g.intensity(), g2.intensity());
                assert_eq!(g.phase(), g2.phase());
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
