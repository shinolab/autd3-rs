use std::num::NonZeroUsize;

use autd3_gain_holo::NalgebraBackend;

use crate::{
    AUTDProtoBufError,
    pb::*,
    to_holo,
    traits::{FromMessage, driver::datagram::gain::IntoLightweightGain},
};
use autd3_core::acoustics::directivity::Sphere;

impl From<autd3_gain_holo::LMOption<Sphere>> for LmOption {
    fn from(value: autd3_gain_holo::LMOption<Sphere>) -> Self {
        Self {
            eps_1: Some(value.eps_1),
            eps_2: Some(value.eps_2),
            tau: Some(value.tau),
            k_max: Some(value.k_max.get() as _),
            initial: value.initial,
            constraint: Some(value.constraint.into()),
        }
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

impl IntoLightweightGain
    for autd3_gain_holo::LM<
        autd3_core::acoustics::directivity::Sphere,
        NalgebraBackend<autd3_core::acoustics::directivity::Sphere>,
    >
{
    fn into_lightweight(self) -> Gain {
        Gain {
            gain: Some(gain::Gain::Lm(Lm {
                holo: to_holo!(self),
                option: Some(self.option.into()),
            })),
        }
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
    use crate::DatagramLightweight;

    use super::*;
    use autd3_core::geometry::Point3;
    use rand::Rng;

    #[test]
    fn test_holo_lm() {
        let mut rng = rand::rng();

        let foci = vec![
            (
                Point3::new(rng.random(), rng.random(), rng.random()),
                rng.random::<f32>() * autd3_gain_holo::Pa,
            ),
            (
                Point3::new(rng.random(), rng.random(), rng.random()),
                rng.random::<f32>() * autd3_gain_holo::Pa,
            ),
        ];
        let option = autd3_gain_holo::LMOption {
            eps_1: rng.random::<f32>(),
            eps_2: rng.random::<f32>(),
            tau: rng.random::<f32>(),
            k_max: NonZeroUsize::new(rng.random_range(1..10)).unwrap(),
            initial: vec![rng.random::<f32>(), rng.random::<f32>()],
            ..Default::default()
        };
        let holo = autd3_gain_holo::LM {
            foci: foci.clone(),
            option: option.clone(),
            backend: std::sync::Arc::new(NalgebraBackend::default()),
        };
        let msg = holo.into_datagram_lightweight(None).unwrap();
        match msg.datagram {
            Some(datagram::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Lm(g)),
                ..
            })) => {
                let holo2 = autd3_gain_holo::LM::from_msg(g).unwrap();
                assert_eq!(option, holo2.option);
                foci.iter().zip(holo2.foci.iter()).for_each(|(f1, f2)| {
                    assert_eq!(f1, f2);
                });
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
