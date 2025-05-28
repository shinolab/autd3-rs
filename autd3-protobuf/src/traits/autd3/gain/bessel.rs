use crate::{
    AUTDProtoBufError,
    pb::*,
    traits::{FromMessage, driver::datagram::gain::IntoLightweightGain},
};

impl From<autd3::gain::BesselOption> for BesselOption {
    fn from(value: autd3::gain::BesselOption) -> Self {
        Self {
            intensity: Some(value.intensity.into()),
            phase_offset: Some(value.phase_offset.into()),
        }
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

impl IntoLightweightGain for autd3::gain::Bessel {
    fn into_lightweight(self) -> Gain {
        Gain {
            gain: Some(gain::Gain::Bessel(Bessel {
                pos: Some(self.pos.into()),
                dir: Some(self.dir.into()),
                theta: Some(self.theta.into()),
                option: Some(self.option.into()),
            })),
        }
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
            theta: autd3_core::common::Angle::from_msg(
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
    use crate::DatagramLightweight;

    use super::*;
    use autd3::prelude::Phase;
    use autd3_driver::{
        common::rad,
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
        let msg = g.clone().into_datagram_lightweight(None).unwrap();

        match msg.datagram {
            Some(raw_datagram::Datagram::Gain(Gain {
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
                assert_eq!(g.option, g2.option);
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
