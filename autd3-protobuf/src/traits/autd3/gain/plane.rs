use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3::gain::Plane {
    type Message = DatagramLightweight;

    #[allow(clippy::unnecessary_cast)]
    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram_lightweight::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Plane(Plane {
                    intensity: Some(self.intensity().to_msg(None)),
                    dir: Some(self.dir().to_msg(None)),
                    phase_offset: Some(self.phase_offset().to_msg(None)),
                })),
                segment: Segment::S0 as _,
                update_segment: true,
            })),
        }
    }
}

impl ToMessage for autd3_driver::datagram::DatagramWithSegment<autd3::gain::Plane> {
    type Message = DatagramLightweight;

    #[allow(clippy::unnecessary_cast)]
    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram_lightweight::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Plane(Plane {
                    intensity: Some(self.intensity().to_msg(None)),
                    dir: Some(self.dir().to_msg(None)),
                    phase_offset: Some(self.phase_offset().to_msg(None)),
                })),
                segment: self.segment() as _,
                update_segment: self.update_segment(),
            })),
        }
    }
}

impl FromMessage<Plane> for autd3::gain::Plane {
    #[allow(clippy::unnecessary_cast)]
    fn from_msg(msg: &Plane) -> Option<Self> {
        Some(
            Self::new(autd3_driver::geometry::Vector3::from_msg(
                msg.dir.as_ref()?,
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
    use autd3_driver::{
        fpga::{EmitIntensity, Phase},
        geometry::Vector3,
    };
    use rand::Rng;

    #[test]
    fn test_phase() {
        let mut rng = rand::thread_rng();

        let g = autd3::gain::Plane::new(Vector3::new(rng.gen(), rng.gen(), rng.gen()))
            .with_intensity(EmitIntensity::new(rng.gen()))
            .with_phase(Phase::new(rng.gen()));
        let msg = g.to_msg(None);

        match msg.datagram {
            Some(datagram_lightweight::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Plane(gain)),
            })) => {
                let g2 = autd3::gain::Plane::from_msg(&gain).unwrap();
                assert_approx_eq::assert_approx_eq!(g.dir().x, g2.dir().x);
                assert_approx_eq::assert_approx_eq!(g.dir().y, g2.dir().y);
                assert_approx_eq::assert_approx_eq!(g.dir().z, g2.dir().z);
                assert_eq!(g.intensity(), g2.intensity());
                assert_eq!(g.phase(), g2.phase());
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
