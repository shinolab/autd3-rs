use autd3_gain_holo::{LinAlgBackend, NalgebraBackend};

use crate::{
    pb::*,
    to_holo,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3_gain_holo::Naive<NalgebraBackend> {
    type Message = DatagramLightweight;

    #[allow(clippy::unnecessary_cast)]
    fn to_msg(&self) -> Self::Message {
        Self::Message {
            datagram: Some(datagram_lightweight::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Naive(Naive {
                    holo: to_holo!(self),
                    constraint: Some(self.constraint().to_msg()),
                })),
            })),
        }
    }
}

impl FromMessage<Naive> for autd3_gain_holo::Naive<NalgebraBackend> {
    #[allow(clippy::unnecessary_cast)]
    fn from_msg(msg: &Naive) -> Option<Self> {
        Some(
            Self::new(NalgebraBackend::new().ok()?)
                .with_constraint(autd3_gain_holo::EmissionConstraint::from_msg(
                    msg.constraint.as_ref()?,
                )?)
                .add_foci_from_iter(
                    msg.holo
                        .iter()
                        .map(|h| {
                            Some((
                                autd3_driver::geometry::Vector3::from_msg(h.pos.as_ref()?)?,
                                h.amp.as_ref()?.value as autd3_driver::defined::float
                                    * autd3_gain_holo::Pascal,
                            ))
                        })
                        .collect::<Option<Vec<_>>>()?,
                ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3_driver::geometry::Vector3;
    use rand::Rng;

    #[test]
    fn test_holo_naive() {
        let mut rng = rand::thread_rng();

        let holo = autd3_gain_holo::Naive::new(NalgebraBackend::new().unwrap())
            .add_focus(
                Vector3::new(rng.gen(), rng.gen(), rng.gen()),
                rng.gen::<autd3_driver::defined::float>() * autd3_gain_holo::Pascal,
            )
            .add_focus(
                Vector3::new(rng.gen(), rng.gen(), rng.gen()),
                rng.gen::<autd3_driver::defined::float>() * autd3_gain_holo::Pascal,
            );
        let msg = holo.to_msg();

        match msg.datagram {
            Some(datagram_lightweight::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Naive(g)),
            })) => {
                let holo2 = autd3_gain_holo::Naive::from_msg(&g).unwrap();
                assert_eq!(holo.constraint(), holo2.constraint());
                holo.foci().zip(holo2.foci()).for_each(|(f1, f2)| {
                    assert_approx_eq::assert_approx_eq!(f1.0.x, f2.0.x);
                    assert_approx_eq::assert_approx_eq!(f1.0.y, f2.0.y);
                    assert_approx_eq::assert_approx_eq!(f1.0.z, f2.0.z);
                    assert_approx_eq::assert_approx_eq!(f1.1.as_pascal(), f2.1.as_pascal());
                });
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}