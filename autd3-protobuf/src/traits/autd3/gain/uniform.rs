use crate::{
    AUTDProtoBufError,
    pb::*,
    traits::{FromMessage, ToMessage, driver::datagram::gain::IntoLightweightGain},
};

impl IntoLightweightGain for autd3::gain::Uniform {
    fn into_lightweight(&self) -> Gain {
        Gain {
            gain: Some(gain::Gain::Uniform(Uniform {
                intensity: Some(self.intensity.to_msg(None).unwrap()),
                phase: Some(self.phase.to_msg(None).unwrap()),
            })),
        }
    }
}

impl FromMessage<Uniform> for autd3::gain::Uniform {
    fn from_msg(msg: Uniform) -> Result<Self, AUTDProtoBufError> {
        Ok(Self {
            phase: autd3_driver::firmware::fpga::Phase::from_msg(
                msg.phase.ok_or(AUTDProtoBufError::DataParseError)?,
            )?,
            intensity: autd3_driver::firmware::fpga::EmitIntensity::from_msg(
                msg.intensity.ok_or(AUTDProtoBufError::DataParseError)?,
            )?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3_driver::firmware::fpga::{EmitIntensity, Phase};
    use rand::Rng;

    #[test]
    fn test_phase() {
        let mut rng = rand::rng();

        let g = autd3::gain::Uniform {
            intensity: EmitIntensity(rng.random()),
            phase: Phase(rng.random()),
        };
        let msg = g.to_msg(None).unwrap();
        match msg.datagram {
            Some(datagram::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Uniform(gain)),
                ..
            })) => {
                let g2 = autd3::gain::Uniform::from_msg(gain).unwrap();
                assert_eq!(g.intensity, g2.intensity);
                assert_eq!(g.phase, g2.phase);
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
