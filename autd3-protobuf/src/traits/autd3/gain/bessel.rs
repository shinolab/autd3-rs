use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3::gain::Bessel {
    type Message = DatagramLightweight;

    #[allow(clippy::unnecessary_cast)]
    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram_lightweight::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Bessel(Bessel {
                    intensity: Some(self.intensity().to_msg(None)),
                    pos: Some(self.pos().to_msg(None)),
                    dir: Some(self.dir().to_msg(None)),
                    theta: self.theta() as _,
                    phase: Some(self.phase().to_msg(None)),
                })),
            })),
        }
    }
}

impl FromMessage<Bessel> for autd3::gain::Bessel {
    #[allow(clippy::unnecessary_cast)]
    fn from_msg(msg: &Bessel) -> Option<Self> {
        Some(
            Self::new(
                autd3_driver::geometry::Vector3::from_msg(msg.pos.as_ref()?)?,
                autd3_driver::geometry::Vector3::from_msg(msg.dir.as_ref()?)?,
                msg.theta as _,
            )
            .with_intensity(autd3_driver::common::EmitIntensity::from_msg(
                msg.intensity.as_ref()?,
            )?)
            .with_phase(autd3_driver::common::Phase::from_msg(msg.phase.as_ref()?)?),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3_driver::{common::EmitIntensity, geometry::Vector3};
    use rand::Rng;

    #[test]
    fn test_bessel() {
        let mut rng = rand::thread_rng();

        let g = autd3::gain::Bessel::new(
            Vector3::new(rng.gen(), rng.gen(), rng.gen()),
            Vector3::new(rng.gen(), rng.gen(), rng.gen()),
            rng.gen(),
        )
        .with_intensity(EmitIntensity::new(rng.gen()));
        let msg = g.to_msg(None);

        match msg.datagram {
            Some(datagram_lightweight::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Bessel(gain)),
            })) => {
                let g2 = autd3::gain::Bessel::from_msg(&gain).unwrap();
                assert_approx_eq::assert_approx_eq!(g.pos().x, g2.pos().x);
                assert_approx_eq::assert_approx_eq!(g.pos().y, g2.pos().y);
                assert_approx_eq::assert_approx_eq!(g.pos().z, g2.pos().z);
                assert_approx_eq::assert_approx_eq!(g.dir().x, g2.dir().x);
                assert_approx_eq::assert_approx_eq!(g.dir().y, g2.dir().y);
                assert_approx_eq::assert_approx_eq!(g.dir().z, g2.dir().z);
                assert_approx_eq::assert_approx_eq!(g.theta(), g2.theta());
                assert_eq!(g.intensity(), g2.intensity());
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
