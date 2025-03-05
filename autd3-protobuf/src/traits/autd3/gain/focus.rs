use crate::{
    AUTDProtoBufError,
    pb::*,
    traits::{FromMessage, ToMessage, driver::datagram::gain::IntoLightweightGain},
};

impl ToMessage for autd3::gain::FocusOption {
    type Message = FocusOption;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(Self::Message {
            intensity: Some(self.intensity.to_msg(None)?),
            phase_offset: Some(self.phase_offset.to_msg(None)?),
        })
    }
}

impl FromMessage<FocusOption> for autd3::gain::FocusOption {
    fn from_msg(msg: FocusOption) -> Result<Self, AUTDProtoBufError> {
        let default = autd3::gain::FocusOption::default();
        Ok(Self {
            intensity: msg
                .intensity
                .map(autd3_driver::firmware::fpga::EmitIntensity::from_msg)
                .transpose()?
                .unwrap_or(default.intensity),
            phase_offset: msg
                .phase_offset
                .map(autd3_driver::firmware::fpga::Phase::from_msg)
                .transpose()?
                .unwrap_or(default.phase_offset),
        })
    }
}

impl IntoLightweightGain for autd3::gain::Focus {
    fn into_lightweight(&self) -> Gain {
        Gain {
            gain: Some(gain::Gain::Focus(Focus {
                pos: Some(self.pos.to_msg(None).unwrap()),
                option: Some(self.option.to_msg(None).unwrap()),
            })),
        }
    }
}

impl FromMessage<Focus> for autd3::gain::Focus {
    fn from_msg(msg: Focus) -> Result<Self, AUTDProtoBufError> {
        Ok(Self {
            pos: autd3_core::geometry::Point3::from_msg(
                msg.pos.ok_or(AUTDProtoBufError::DataParseError)?,
            )?,
            option: autd3::gain::FocusOption::from_msg(
                msg.option.ok_or(AUTDProtoBufError::DataParseError)?,
            )?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3::prelude::Phase;
    use autd3_driver::{firmware::fpga::EmitIntensity, geometry::Point3};
    use rand::Rng;

    #[test]
    fn focus() {
        let mut rng = rand::rng();

        let g = autd3::gain::Focus {
            pos: Point3::new(rng.random(), rng.random(), rng.random()),
            option: autd3::gain::FocusOption {
                intensity: EmitIntensity(rng.random()),
                phase_offset: Phase(rng.random()),
            },
        };
        let msg = g.to_msg(None).unwrap();

        match msg.datagram {
            Some(datagram::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Focus(gain)),
                ..
            })) => {
                let g2 = autd3::gain::Focus::from_msg(gain).unwrap();
                assert_eq!(g.pos.x, g2.pos.x);
                assert_eq!(g.pos.y, g2.pos.y);
                assert_eq!(g.pos.z, g2.pos.z);
                assert_eq!(g.option.intensity, g2.option.intensity);
                assert_eq!(g.option.phase_offset, g2.option.phase_offset);
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
