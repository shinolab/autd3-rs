use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

impl ToMessage for autd3::gain::Plane {
    type Message = Datagram;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(Self::Message {
            datagram: Some(datagram::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Plane(Plane {
                    dir: Some(self.dir.to_msg(None)?),
                    intensity: Some(self.option.intensity.to_msg(None)?),
                    phase_offset: Some(self.option.phase_offset.to_msg(None)?),
                })),
            })),
        })
    }
}

impl FromMessage<Plane> for autd3::gain::Plane {
    fn from_msg(msg: &Plane) -> Result<Self, AUTDProtoBufError> {
        Ok(Self {
            dir: autd3_core::geometry::UnitVector3::from_msg(&msg.dir)?,
            option: autd3::gain::PlaneOption {
                intensity: msg
                    .intensity
                    .as_ref()
                    .map(autd3_driver::firmware::fpga::EmitIntensity::from_msg)
                    .transpose()?
                    .unwrap_or(autd3::gain::PlaneOption::default().intensity),
                phase_offset: msg
                    .phase_offset
                    .as_ref()
                    .map(autd3_driver::firmware::fpga::Phase::from_msg)
                    .transpose()?
                    .unwrap_or(autd3::gain::PlaneOption::default().phase_offset),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3_driver::{
        firmware::fpga::{EmitIntensity, Phase},
        geometry::{UnitVector3, Vector3},
    };
    use rand::Rng;

    #[test]
    fn test_phase() {
        let mut rng = rand::rng();

        let g = autd3::gain::Plane {
            dir: UnitVector3::new_normalize(Vector3::new(rng.random(), rng.random(), rng.random())),
            option: autd3::gain::PlaneOption {
                intensity: EmitIntensity(rng.random()),
                phase_offset: Phase(rng.random()),
            },
        };
        let msg = g.to_msg(None).unwrap();
        match msg.datagram {
            Some(datagram::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Plane(gain)),
                ..
            })) => {
                let g2 = autd3::gain::Plane::from_msg(&gain).unwrap();
                assert_eq!(g.dir.x, g2.dir.x);
                assert_eq!(g.dir.y, g2.dir.y);
                assert_eq!(g.dir.z, g2.dir.z);
                assert_eq!(g.option.intensity, g2.option.intensity);
                assert_eq!(g.option.phase_offset, g2.option.phase_offset);
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
