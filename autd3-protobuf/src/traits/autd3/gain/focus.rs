use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

impl ToMessage for autd3::gain::Focus {
    type Message = Datagram;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Focus(Focus {
                    intensity: Some(self.intensity().to_msg(None)),
                    pos: Some(self.pos().to_msg(None)),
                    phase_offset: Some(self.phase_offset().to_msg(None)),
                })),
            })),
            timeout: None,
            parallel_threshold: None,
        }
    }
}

impl FromMessage<Focus> for autd3::gain::Focus {
    fn from_msg(msg: &Focus) -> Result<Self, AUTDProtoBufError> {
        let mut g = Self::new(autd3_driver::geometry::Point3::from_msg(&msg.pos)?);
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
    use autd3_driver::{firmware::fpga::EmitIntensity, geometry::Point3};
    use rand::Rng;

    #[test]
    fn focus() {
        let mut rng = rand::thread_rng();

        let g = autd3::gain::Focus::new(Point3::new(rng.gen(), rng.gen(), rng.gen()))
            .with_intensity(EmitIntensity::new(rng.gen()));
        let msg = g.to_msg(None);

        match msg.datagram {
            Some(datagram::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Focus(gain)),
                ..
            })) => {
                let g2 = autd3::gain::Focus::from_msg(&gain).unwrap();
                approx::assert_abs_diff_eq!(g.pos().x, g2.pos().x);
                approx::assert_abs_diff_eq!(g.pos().y, g2.pos().y);
                approx::assert_abs_diff_eq!(g.pos().z, g2.pos().z);
                assert_eq!(g.intensity(), g2.intensity());
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
