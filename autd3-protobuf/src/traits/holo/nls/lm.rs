use std::num::NonZeroUsize;

use autd3_gain_holo::NalgebraBackend;

use crate::{
    AUTDProtoBufError,
    pb::*,
    to_holo,
    traits::{FromMessage, ToMessage},
};
use autd3_core::acoustics::directivity::Sphere;

impl ToMessage for autd3_gain_holo::LMOption<Sphere> {
    type Message = LmOption;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(Self::Message {
            eps_1: Some(self.eps_1 as _),
            eps_2: Some(self.eps_2 as _),
            tau: Some(self.tau as _),
            k_max: Some(self.k_max.get() as _),
            initial: self.initial.to_vec(),
            constraint: Some(self.constraint.to_msg(None)?),
        })
    }
}

impl FromMessage<LmOption> for autd3_gain_holo::LMOption<Sphere> {
    fn from_msg(msg: LmOption) -> Result<Self, AUTDProtoBufError> {
        let default = autd3_gain_holo::LMOption::<Sphere>::default();
        Ok(Self {
            eps_1: msg.eps_1.unwrap_or(default.eps_1),
            eps_2: msg.eps_2.unwrap_or(default.eps_2),
            tau: msg.tau.unwrap_or(default.tau),
            k_max: msg
                .k_max
                .map(usize::try_from)
                .transpose()?
                .map(NonZeroUsize::try_from)
                .transpose()?
                .unwrap_or(default.k_max),
            constraint: msg
                .constraint
                .map(autd3_gain_holo::EmissionConstraint::from_msg)
                .transpose()?
                .unwrap_or(default.constraint),
            initial: msg.initial,
            __phantom: std::marker::PhantomData,
        })
    }
}

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
                    option: Some(self.option.to_msg(None)?),
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
    fn from_msg(msg: Lm) -> Result<Self, AUTDProtoBufError> {
        Ok(Self {
            foci: msg
                .holo
                .into_iter()
                .map(|h| {
                    Ok((
                        autd3_core::geometry::Point3::from_msg(
                            h.pos.ok_or(AUTDProtoBufError::DataParseError)?,
                        )?,
                        autd3_gain_holo::Amplitude::from_msg(
                            h.amp.ok_or(AUTDProtoBufError::DataParseError)?,
                        )?,
                    ))
                })
                .collect::<Result<Vec<_>, AUTDProtoBufError>>()?,
            option: autd3_gain_holo::LMOption::from_msg(
                msg.option.ok_or(AUTDProtoBufError::DataParseError)?,
            )?,
            backend: std::sync::Arc::new(NalgebraBackend::default()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3_core::geometry::Point3;
    use rand::Rng;

    #[test]
    fn test_holo_lm() {
        let mut rng = rand::rng();

        let holo = autd3_gain_holo::LM {
            foci: vec![
                (
                    Point3::new(rng.random(), rng.random(), rng.random()),
                    rng.random::<f32>() * autd3_gain_holo::Pa,
                ),
                (
                    Point3::new(rng.random(), rng.random(), rng.random()),
                    rng.random::<f32>() * autd3_gain_holo::Pa,
                ),
            ],
            option: autd3_gain_holo::LMOption {
                eps_1: rng.random::<f32>(),
                eps_2: rng.random::<f32>(),
                tau: rng.random::<f32>(),
                k_max: NonZeroUsize::new(rng.random_range(1..10)).unwrap(),
                initial: vec![rng.random::<f32>(), rng.random::<f32>()],
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
                let holo2 = autd3_gain_holo::LM::from_msg(g).unwrap();
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
