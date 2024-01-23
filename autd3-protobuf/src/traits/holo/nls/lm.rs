use autd3_gain_holo::{LinAlgBackend, NalgebraBackend};

use crate::{
    pb::*,
    to_holo,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3_gain_holo::LM<NalgebraBackend> {
    type Message = DatagramLightweight;

    #[allow(clippy::unnecessary_cast)]
    fn to_msg(&self) -> Self::Message {
        Self::Message {
            datagram: Some(datagram_lightweight::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Lm(Lm {
                    holo: to_holo!(self),
                    eps_1: self.eps_1() as _,
                    eps_2: self.eps_2() as _,
                    tau: self.tau() as _,
                    k_max: self.k_max() as _,
                    initial: self.initial().iter().map(|&v| v as _).collect(),
                    constraint: Some(self.constraint().to_msg()),
                })),
            })),
        }
    }
}

impl FromMessage<Lm> for autd3_gain_holo::LM<NalgebraBackend> {
    #[allow(clippy::unnecessary_cast)]
    fn from_msg(msg: &Lm) -> Option<Self> {
        Some(
            Self::new(NalgebraBackend::new().ok()?)
                .with_eps_1(msg.eps_1 as _)
                .with_eps_2(msg.eps_2 as _)
                .with_tau(msg.tau as _)
                .with_k_max(msg.k_max as _)
                .with_initial(msg.initial.iter().map(|&v| v as _).collect())
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
    fn test_holo_sdp() {
        let mut rng = rand::thread_rng();

        let holo = autd3_gain_holo::LM::new(NalgebraBackend::new().unwrap())
            .with_eps_1(rng.gen())
            .with_eps_2(rng.gen())
            .with_tau(rng.gen())
            .with_k_max(rng.gen())
            .with_initial(vec![rng.gen(), rng.gen(), rng.gen()])
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
                gain: Some(gain::Gain::Lm(g)),
            })) => {
                let holo2 = autd3_gain_holo::LM::from_msg(&g).unwrap();
                assert_approx_eq::assert_approx_eq!(holo.eps_1(), holo2.eps_1());
                assert_approx_eq::assert_approx_eq!(holo.eps_2(), holo2.eps_2());
                assert_approx_eq::assert_approx_eq!(holo.tau(), holo2.tau());
                assert_eq!(holo.k_max(), holo2.k_max());
                holo.initial()
                    .iter()
                    .zip(holo2.initial().iter())
                    .for_each(|(v1, v2)| {
                        assert_approx_eq::assert_approx_eq!(v1, v2);
                    });
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
