use crate::{
    AUTDProtoBufError,
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3::gain::PlaneOption {
    type Message = PlaneOption;

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

impl FromMessage<PlaneOption> for autd3::gain::PlaneOption {
    fn from_msg(msg: PlaneOption) -> Result<Self, AUTDProtoBufError> {
        let default = autd3::gain::PlaneOption::default();
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
                    option: Some(self.option.to_msg(None)?),
                })),
            })),
        })
    }
}

impl FromMessage<Plane> for autd3::gain::Plane {
    fn from_msg(msg: Plane) -> Result<Self, AUTDProtoBufError> {
        Ok(Self {
            dir: autd3_core::geometry::UnitVector3::from_msg(
                msg.dir.ok_or(AUTDProtoBufError::DataParseError)?,
            )?,
            option: autd3::gain::PlaneOption::from_msg(
                msg.option.ok_or(AUTDProtoBufError::DataParseError)?,
            )?,
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
                let g2 = autd3::gain::Plane::from_msg(gain).unwrap();
                approx::assert_abs_diff_eq!(g.dir.x, g2.dir.x);
                approx::assert_abs_diff_eq!(g.dir.y, g2.dir.y);
                approx::assert_abs_diff_eq!(g.dir.z, g2.dir.z);
                assert_eq!(g.option.intensity, g2.option.intensity);
                assert_eq!(g.option.phase_offset, g2.option.phase_offset);
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
