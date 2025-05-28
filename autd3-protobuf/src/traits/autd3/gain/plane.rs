use crate::{
    AUTDProtoBufError,
    pb::*,
    traits::{FromMessage, driver::datagram::gain::IntoLightweightGain},
};

impl From<autd3::gain::PlaneOption> for PlaneOption {
    fn from(value: autd3::gain::PlaneOption) -> Self {
        Self {
            intensity: Some(value.intensity.into()),
            phase_offset: Some(value.phase_offset.into()),
        }
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

impl IntoLightweightGain for autd3::gain::Plane {
    fn into_lightweight(self) -> Gain {
        Gain {
            gain: Some(gain::Gain::Plane(Plane {
                dir: Some(self.dir.into()),
                option: Some(self.option.into()),
            })),
        }
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
    use crate::DatagramLightweight;

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
        let msg = g.clone().into_datagram_lightweight(None).unwrap();
        match msg.datagram {
            Some(raw_datagram::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Plane(gain)),
                ..
            })) => {
                let g2 = autd3::gain::Plane::from_msg(gain).unwrap();
                approx::assert_abs_diff_eq!(g.dir.x, g2.dir.x);
                approx::assert_abs_diff_eq!(g.dir.y, g2.dir.y);
                approx::assert_abs_diff_eq!(g.dir.z, g2.dir.z);
                assert_eq!(g.option, g2.option);
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
