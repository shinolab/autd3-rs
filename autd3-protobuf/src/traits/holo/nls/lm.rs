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
        autd3_core::acoustics::directivity::Sphere,
        NalgebraBackend<autd3_core::acoustics::directivity::Sphere>,
    >
{
    type Message = Datagram;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(Self::Message {
            datagram: Some(datagram::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Lm(Lm {
                    holo: to_holo!(self),
                    eps_1: Some(self.option.eps_1 as _),
                    eps_2: Some(self.option.eps_2 as _),
                    tau: Some(self.option.tau as _),
                    k_max: Some(self.option.k_max.get() as _),
                    initial: self.option.initial.iter().map(|&v| v as _).collect(),
                    constraint: Some(self.option.constraint.to_msg(None)?),
                })),
            })),
        })
    }
}

impl FromMessage<Lm>
    for autd3_gain_holo::LM<
        autd3_core::acoustics::directivity::Sphere,
        NalgebraBackend<autd3_core::acoustics::directivity::Sphere>,
    >
{
    fn from_msg(msg: &Lm) -> Result<Self, AUTDProtoBufError> {
        Ok(
            Self {
                foci: msg
                    .holo
                    .iter()
                    .map(|h| {
                        Ok((
                            autd3_core::geometry::Point3::from_msg(&h.pos)?,
                            autd3_gain_holo::Amplitude::from_msg(&h.amp)?,
                        ))
                    })
                    .collect::<Result<Vec<_>, AUTDProtoBufError>>()?,
                option:
                    autd3_gain_holo::LMOption {
                        eps_1:
                            msg.eps_1.unwrap_or(
                                autd3_gain_holo::LMOption::<
                                    autd3_core::acoustics::directivity::Sphere,
                                >::default()
                                .eps_1,
                            ),
                        eps_2:
                            msg.eps_2.unwrap_or(
                                autd3_gain_holo::LMOption::<
                                    autd3_core::acoustics::directivity::Sphere,
                                >::default()
                                .eps_2,
                            ),
                        tau:
                            msg.tau.unwrap_or(
                                autd3_gain_holo::LMOption::<
                                    autd3_core::acoustics::directivity::Sphere,
                                >::default()
                                .tau,
                            ),
                        k_max: msg
                            .k_max
                            .map(usize::try_from)
                            .transpose()?
                            .map(|x| NonZeroUsize::new(x).ok_or(AUTDProtoBufError::DataParseError))
                            .transpose()?
                            .unwrap_or(
                                autd3_gain_holo::LMOption::<
                                    autd3_core::acoustics::directivity::Sphere,
                                >::default()
                                .k_max,
                            ),
                        initial: msg.initial.clone(),
                        constraint: msg
                            .constraint
                            .as_ref()
                            .map(autd3_gain_holo::EmissionConstraint::from_msg)
                            .transpose()?
                            .unwrap_or(
                                autd3_gain_holo::LMOption::<
                                    autd3_core::acoustics::directivity::Sphere,
                                >::default()
                                .constraint,
                            ),
                        ..Default::default()
                    },
                backend: std::sync::Arc::new(NalgebraBackend::default()),
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3_core::geometry::Point3;
    use rand::Rng;

    #[test]
    fn test_holo_lm() {
        let mut rng = rand::thread_rng();

        let holo = autd3_gain_holo::LM {
            foci: vec![
                (
                    Point3::new(rng.gen(), rng.gen(), rng.gen()),
                    rng.gen::<f32>() * autd3_gain_holo::Pa,
                ),
                (
                    Point3::new(rng.gen(), rng.gen(), rng.gen()),
                    rng.gen::<f32>() * autd3_gain_holo::Pa,
                ),
            ],
            option: autd3_gain_holo::LMOption {
                eps_1: rng.gen::<f32>(),
                eps_2: rng.gen::<f32>(),
                tau: rng.gen::<f32>(),
                k_max: NonZeroUsize::new(rng.gen_range(1..10)).unwrap(),
                initial: vec![rng.gen::<f32>(), rng.gen::<f32>()],
                ..Default::default()
            },
            backend: std::sync::Arc::new(NalgebraBackend::default()),
        };
        let msg = holo.to_msg(None).unwrap();
        match msg.datagram {
            Some(datagram::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Lm(g)),
                ..
            })) => {
                let holo2 = autd3_gain_holo::LM::from_msg(&g).unwrap();
                assert_eq!(holo.option.eps_1, holo2.option.eps_1);
                assert_eq!(holo.option.eps_2, holo2.option.eps_2);
                assert_eq!(holo.option.tau, holo2.option.tau);
                assert_eq!(holo.option.k_max, holo2.option.k_max);
                assert_eq!(holo.option.initial, holo2.option.initial);
                assert_eq!(holo.option.constraint, holo2.option.constraint);
                holo.foci
                    .iter()
                    .zip(holo2.foci.iter())
                    .for_each(|(f1, f2)| {
                        approx::assert_abs_diff_eq!(f1.1.pascal(), f2.1.pascal());
                        approx::assert_abs_diff_eq!(f1.0.x, f2.0.x);
                        approx::assert_abs_diff_eq!(f1.0.y, f2.0.y);
                        approx::assert_abs_diff_eq!(f1.0.z, f2.0.z);
                    });
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
