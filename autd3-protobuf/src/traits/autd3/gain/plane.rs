use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

impl ToMessage for autd3::gain::Plane {
    type Message = Datagram;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Plane(Plane {
                    intensity: Some(self.intensity().to_msg(None)),
                    dir: Some(self.dir().to_msg(None)),
                    phase_offset: Some(self.phase_offset().to_msg(None)),
                })),
            })),
            timeout: None,
            parallel_threshold: None,
        }
    }
}

impl FromMessage<Plane> for autd3::gain::Plane {
    fn from_msg(msg: &Plane) -> Result<Self, AUTDProtoBufError> {
        let mut g = Self::new(autd3_driver::geometry::UnitVector3::new_normalize(
            autd3_driver::geometry::Vector3::from_msg(&msg.dir)?,
        ));
        if let Some(intensity) = msg.intensity.as_ref() {
            g = g.with_intensity(autd3_driver::firmware::fpga::EmitIntensity::from_msg(
                intensity,
            )?);
        }
        if let Some(phase_offset) = msg.phase_offset.as_ref() {
            g = g.with_phase_offset(autd3_driver::firmware::fpga::Phase::from_msg(phase_offset)?);
        }
        Ok(g)
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
        let mut rng = rand::thread_rng();

        let g = autd3::gain::Plane::new(UnitVector3::new_normalize(Vector3::new(
            rng.gen(),
            rng.gen(),
            rng.gen(),
        )))
        .with_intensity(EmitIntensity::new(rng.gen()))
        .with_phase_offset(Phase::new(rng.gen()));
        let msg = g.to_msg(None);

        match msg.datagram {
            Some(datagram::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Plane(gain)),
                ..
            })) => {
                let g2 = autd3::gain::Plane::from_msg(&gain).unwrap();
                approx::assert_abs_diff_eq!(g.dir().x, g2.dir().x);
                approx::assert_abs_diff_eq!(g.dir().y, g2.dir().y);
                approx::assert_abs_diff_eq!(g.dir().z, g2.dir().z);
                assert_eq!(g.intensity(), g2.intensity());
                assert_eq!(g.phase_offset(), g2.phase_offset());
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
