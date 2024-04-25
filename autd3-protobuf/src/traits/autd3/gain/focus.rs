use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3::gain::Focus {
    type Message = DatagramLightweight;

    #[allow(clippy::unnecessary_cast)]
    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram_lightweight::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Focus(Focus {
                    intensity: Some(self.intensity().to_msg(None)),
                    pos: Some(self.pos().to_msg(None)),
                    phase_offset: Some(self.phase_offset().to_msg(None)),
                })),
                segment: Segment::S0 as _,
                transition_mode: Some(TransitionMode::SyncIdx.into()),
                transition_value: Some(0),
            })),
        }
    }
}

impl ToMessage for autd3_driver::datagram::DatagramWithSegment<autd3::gain::Focus> {
    type Message = DatagramLightweight;

    #[allow(clippy::unnecessary_cast)]
    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram_lightweight::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Focus(Focus {
                    intensity: Some(self.intensity().to_msg(None)),
                    pos: Some(self.pos().to_msg(None)),
                    phase_offset: Some(self.phase_offset().to_msg(None)),
                })),
                segment: self.segment() as _,
                transition_mode: self.transition_mode().map(|m| m.mode() as _),
                transition_value: self.transition_mode().map(|m| m.value()),
            })),
        }
    }
}

impl FromMessage<Focus> for autd3::gain::Focus {
    #[allow(clippy::unnecessary_cast)]
    fn from_msg(msg: &Focus) -> Option<Self> {
        Some(
            Self::new(autd3_driver::geometry::Vector3::from_msg(
                msg.pos.as_ref()?,
            )?)
            .with_intensity(autd3_driver::firmware::fpga::EmitIntensity::from_msg(
                msg.intensity.as_ref()?,
            )?)
            .with_phase_offset(autd3_driver::firmware::fpga::Phase::from_msg(
                msg.phase_offset.as_ref()?,
            )?),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3_driver::{firmware::fpga::EmitIntensity, geometry::Vector3};
    use rand::Rng;

    #[test]
    fn test_focus() {
        let mut rng = rand::thread_rng();

        let g = autd3::gain::Focus::new(Vector3::new(rng.gen(), rng.gen(), rng.gen()))
            .with_intensity(EmitIntensity::new(rng.gen()));
        let msg = g.to_msg(None);

        match msg.datagram {
            Some(datagram_lightweight::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Focus(gain)),
                ..
            })) => {
                let g2 = autd3::gain::Focus::from_msg(&gain).unwrap();
                assert_approx_eq::assert_approx_eq!(g.pos().x, g2.pos().x);
                assert_approx_eq::assert_approx_eq!(g.pos().y, g2.pos().y);
                assert_approx_eq::assert_approx_eq!(g.pos().z, g2.pos().z);
                assert_eq!(g.intensity(), g2.intensity());
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
