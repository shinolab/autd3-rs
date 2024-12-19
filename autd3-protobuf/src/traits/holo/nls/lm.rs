use std::num::NonZeroUsize;

use autd3_gain_holo::NalgebraBackend;

use crate::{
    pb::*,
    to_holo,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

impl ToMessage
    for autd3_gain_holo::LM<
        autd3_driver::acoustics::directivity::Sphere,
        NalgebraBackend<autd3_driver::acoustics::directivity::Sphere>,
    >
{
    type Message = Datagram;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Lm(Lm {
                    holo: to_holo!(self),
                    eps_1: Some(self.eps_1() as _),
                    eps_2: Some(self.eps_2() as _),
                    tau: Some(self.tau() as _),
                    k_max: Some(self.k_max().get() as _),
                    initial: self.initial().iter().map(|&v| v as _).collect(),
                    constraint: Some(self.constraint().to_msg(None)),
                })),
            })),
            timeout: None,
            parallel_threshold: None,
        }
    }
}

impl FromMessage<Lm>
    for autd3_gain_holo::LM<
        autd3_driver::acoustics::directivity::Sphere,
        NalgebraBackend<autd3_driver::acoustics::directivity::Sphere>,
    >
{
    fn from_msg(msg: &Lm) -> Result<Self, AUTDProtoBufError> {
        let mut g = Self::new(
            std::sync::Arc::new(NalgebraBackend::default()),
            msg.holo
                .iter()
                .map(|h| {
                    Ok((
                        autd3_driver::geometry::Point3::from_msg(&h.pos)?,
                        autd3_gain_holo::Amplitude::from_msg(&h.amp)?,
                    ))
                })
                .collect::<Result<Vec<_>, AUTDProtoBufError>>()?,
        )
        .with_initial(msg.initial.clone());
        if let Some(eps_1) = msg.eps_1 {
            g = g.with_eps_1(eps_1 as _);
        }
        if let Some(eps_2) = msg.eps_2 {
            g = g.with_eps_2(eps_2 as _);
        }
        if let Some(tau) = msg.tau {
            g = g.with_tau(tau as _);
        }
        if let Some(k_max) = msg.k_max {
            g = g.with_k_max(
                NonZeroUsize::new(k_max as _).ok_or(AUTDProtoBufError::DataParseError)?,
            );
        }
        if let Some(constraint) = msg.constraint.as_ref() {
            g = g.with_constraint(autd3_gain_holo::EmissionConstraint::from_msg(constraint)?);
        }
        Ok(g)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3_driver::geometry::Point3;
    use rand::Rng;

    #[test]
    fn test_holo_sdp() {
        let mut rng = rand::thread_rng();

        let holo = autd3_gain_holo::LM::new(
            std::sync::Arc::new(NalgebraBackend::default()),
            [
                (
                    Point3::new(rng.gen(), rng.gen(), rng.gen()),
                    rng.gen::<f32>() * autd3_gain_holo::Pa,
                ),
                (
                    Point3::new(rng.gen(), rng.gen(), rng.gen()),
                    rng.gen::<f32>() * autd3_gain_holo::Pa,
                ),
            ],
        )
        .with_eps_1(rng.gen())
        .with_eps_2(rng.gen())
        .with_tau(rng.gen())
        .with_k_max(rng.gen())
        .with_initial(vec![rng.gen(), rng.gen(), rng.gen()]);
        let msg = holo.to_msg(None);

        match msg.datagram {
            Some(datagram::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Lm(g)),
                ..
            })) => {
                let holo2 = autd3_gain_holo::LM::from_msg(&g).unwrap();
                approx::assert_abs_diff_eq!(holo.eps_1(), holo2.eps_1());
                approx::assert_abs_diff_eq!(holo.eps_2(), holo2.eps_2());
                approx::assert_abs_diff_eq!(holo.tau(), holo2.tau());
                assert_eq!(holo.k_max(), holo2.k_max());
                holo.initial()
                    .iter()
                    .zip(holo2.initial().iter())
                    .for_each(|(v1, v2)| {
                        approx::assert_abs_diff_eq!(v1, v2);
                    });
                assert_eq!(holo.constraint(), holo2.constraint());
                holo.foci()
                    .iter()
                    .zip(holo2.foci().iter())
                    .for_each(|(f1, f2)| {
                        approx::assert_abs_diff_eq!(f1.x, f2.x);
                        approx::assert_abs_diff_eq!(f1.y, f2.y);
                        approx::assert_abs_diff_eq!(f1.z, f2.z);
                    });
                holo.amps()
                    .iter()
                    .zip(holo2.amps().iter())
                    .for_each(|(f1, f2)| {
                        approx::assert_abs_diff_eq!(f1.pascal(), f2.pascal());
                    });
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
