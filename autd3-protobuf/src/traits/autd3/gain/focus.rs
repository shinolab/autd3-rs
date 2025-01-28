use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

impl ToMessage for autd3::gain::Focus {
    type Message = Datagram;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(Self::Message {
            datagram: Some(datagram::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Focus(Focus {
                    pos: Some(self.pos.to_msg(None)?),
                    intensity: Some(self.option.intensity.to_msg(None)?),
                    phase_offset: Some(self.option.phase_offset.to_msg(None)?),
                })),
            })),
        })
    }
}

impl FromMessage<Focus> for autd3::gain::Focus {
    fn from_msg(msg: &Focus) -> Result<Self, AUTDProtoBufError> {
        Ok(Self {
            pos: autd3_core::geometry::Point3::from_msg(&msg.pos)?,
            option: autd3::gain::FocusOption {
                intensity: msg
                    .intensity
                    .as_ref()
                    .map(autd3_driver::firmware::fpga::EmitIntensity::from_msg)
                    .transpose()?
                    .unwrap_or(autd3::gain::FocusOption::default().intensity),
                phase_offset: msg
                    .phase_offset
                    .as_ref()
                    .map(autd3_driver::firmware::fpga::Phase::from_msg)
                    .transpose()?
                    .unwrap_or(autd3::gain::FocusOption::default().phase_offset),
            },
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
                let g2 = autd3::gain::Focus::from_msg(&gain).unwrap();
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
