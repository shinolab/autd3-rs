use crate::{
    AUTDProtoBufError,
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3::gain::BesselOption {
    type Message = BesselOption;

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

impl FromMessage<BesselOption> for autd3::gain::BesselOption {
    fn from_msg(msg: BesselOption) -> Result<Self, AUTDProtoBufError> {
        let default = autd3::gain::BesselOption::default();
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

impl ToMessage for autd3::gain::Bessel {
    type Message = Datagram;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(Self::Message {
            datagram: Some(datagram::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Bessel(Bessel {
                    pos: Some(self.pos.to_msg(None)?),
                    dir: Some(self.dir.to_msg(None)?),
                    theta: Some(self.theta.to_msg(None)?),
                    option: Some(self.option.to_msg(None)?),
                })),
            })),
        })
    }
}

impl FromMessage<Bessel> for autd3::gain::Bessel {
    fn from_msg(msg: Bessel) -> Result<Self, AUTDProtoBufError> {
        Ok(Self {
            pos: autd3_core::geometry::Point3::from_msg(
                msg.pos.ok_or(AUTDProtoBufError::DataParseError)?,
            )?,
            dir: autd3_core::geometry::UnitVector3::from_msg(
                msg.dir.ok_or(AUTDProtoBufError::DataParseError)?,
            )?,
            theta: autd3_core::defined::Angle::from_msg(
                msg.theta.ok_or(AUTDProtoBufError::DataParseError)?,
            )?,
            option: autd3::gain::BesselOption::from_msg(
                msg.option.ok_or(AUTDProtoBufError::DataParseError)?,
            )?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3::prelude::Phase;
    use autd3_driver::{
        defined::rad,
        firmware::fpga::EmitIntensity,
        geometry::{Point3, Vector3},
    };
    use rand::Rng;

    #[test]
    fn bessel() {
        let mut rng = rand::rng();

        let g = autd3::gain::Bessel {
            pos: Point3::new(rng.random(), rng.random(), rng.random()),
            dir: autd3_core::geometry::UnitVector3::new_normalize(Vector3::new(
                rng.random(),
                rng.random(),
                rng.random(),
            )),
            theta: rng.random::<f32>() * rad,
            option: autd3::gain::BesselOption {
                intensity: EmitIntensity(rng.random()),
                phase_offset: Phase(rng.random()),
            },
        };
        let msg = g.to_msg(None).unwrap();

        match msg.datagram {
            Some(datagram::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Bessel(gain)),
                ..
            })) => {
                let g2 = autd3::gain::Bessel::from_msg(gain).unwrap();
                assert_eq!(g.pos.x, g2.pos.x);
                assert_eq!(g.pos.y, g2.pos.y);
                assert_eq!(g.pos.z, g2.pos.z);
                approx::assert_abs_diff_eq!(g.dir.x, g2.dir.x);
                approx::assert_abs_diff_eq!(g.dir.y, g2.dir.y);
                approx::assert_abs_diff_eq!(g.dir.z, g2.dir.z);
                assert_eq!(g.theta, g2.theta);
                assert_eq!(g.option.intensity, g2.option.intensity);
                assert_eq!(g.option.phase_offset, g2.option.phase_offset);
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
