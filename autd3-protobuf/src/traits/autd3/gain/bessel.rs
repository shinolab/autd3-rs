use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

impl ToMessage for autd3::gain::Bessel {
    type Message = Datagram;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Bessel(Bessel {
                    intensity: Some(self.intensity().to_msg(None)),
                    pos: Some(self.pos().to_msg(None)),
                    dir: Some(self.dir().to_msg(None)),
                    theta: Some(self.theta().to_msg(None)),
                    phase_offset: Some(self.phase_offset().to_msg(None)),
                })),
            })),
            timeout: None,
            parallel_threshold: None,
        }
    }
}

impl FromMessage<Bessel> for autd3::gain::Bessel {
    fn from_msg(msg: &Bessel) -> Result<Self, AUTDProtoBufError> {
        let mut g = Self::new(
            autd3_driver::geometry::Vector3::from_msg(&msg.pos)?,
            autd3_driver::geometry::UnitVector3::new_normalize(
                autd3_driver::geometry::Vector3::from_msg(&msg.dir)?,
            ),
            autd3_driver::defined::Angle::from_msg(&msg.theta)?,
        );
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
    use autd3_driver::{defined::rad, firmware::fpga::EmitIntensity, geometry::Vector3};
    use rand::Rng;

    #[test]
    fn bessel() {
        let mut rng = rand::thread_rng();

        let g = autd3::gain::Bessel::new(
            Vector3::new(rng.gen(), rng.gen(), rng.gen()),
            autd3_driver::geometry::UnitVector3::new_normalize(Vector3::new(
                rng.gen(),
                rng.gen(),
                rng.gen(),
            )),
            rng.gen::<f32>() * rad,
        )
        .with_intensity(EmitIntensity::new(rng.gen()));
        let msg = g.to_msg(None);

        match msg.datagram {
            Some(datagram::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Bessel(gain)),
                ..
            })) => {
                let g2 = autd3::gain::Bessel::from_msg(&gain).unwrap();
                approx::assert_abs_diff_eq!(g.pos().x, g2.pos().x);
                approx::assert_abs_diff_eq!(g.pos().y, g2.pos().y);
                approx::assert_abs_diff_eq!(g.pos().z, g2.pos().z);
                approx::assert_abs_diff_eq!(g.dir().x, g2.dir().x);
                approx::assert_abs_diff_eq!(g.dir().y, g2.dir().y);
                approx::assert_abs_diff_eq!(g.dir().z, g2.dir().z);
                approx::assert_abs_diff_eq!(g.theta().radian(), g2.theta().radian());
                assert_eq!(g.intensity(), g2.intensity());
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
