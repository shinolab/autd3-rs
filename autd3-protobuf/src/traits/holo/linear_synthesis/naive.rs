use autd3_gain_holo::NalgebraBackend;

use crate::{
    AUTDProtoBufError,
    pb::*,
    to_holo,
    traits::{FromMessage, driver::datagram::gain::IntoLightweightGain},
};
use autd3_core::acoustics::directivity::Sphere;

impl From<autd3_gain_holo::NaiveOption<Sphere>> for NaiveOption {
    fn from(value: autd3_gain_holo::NaiveOption<Sphere>) -> Self {
        Self {
            constraint: Some(value.constraint.into()),
        }
    }
}

impl FromMessage<NaiveOption> for autd3_gain_holo::NaiveOption<Sphere> {
    fn from_msg(msg: NaiveOption) -> Result<Self, AUTDProtoBufError> {
        let default = autd3_gain_holo::NaiveOption::<Sphere>::default();
        Ok(Self {
            constraint: msg
                .constraint
                .map(autd3_gain_holo::EmissionConstraint::from_msg)
                .transpose()?
                .unwrap_or(default.constraint),
            __phantom: std::marker::PhantomData,
        })
    }
}

impl IntoLightweightGain
    for autd3_gain_holo::Naive<
        autd3_core::acoustics::directivity::Sphere,
        NalgebraBackend<autd3_core::acoustics::directivity::Sphere>,
    >
{
    fn into_lightweight(self) -> Gain {
        Gain {
            gain: Some(gain::Gain::Naive(Naive {
                holo: to_holo!(self),
                option: Some(self.option.into()),
            })),
        }
    }
}

impl FromMessage<Naive>
    for autd3_gain_holo::Naive<
        autd3_core::acoustics::directivity::Sphere,
        NalgebraBackend<autd3_core::acoustics::directivity::Sphere>,
    >
{
    fn from_msg(msg: Naive) -> Result<Self, AUTDProtoBufError> {
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
            option: autd3_gain_holo::NaiveOption::from_msg(
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
    fn test_holo_naive() {
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
        let option = autd3_gain_holo::NaiveOption {
            ..Default::default()
        };
        let holo = autd3_gain_holo::Naive {
            foci: foci.clone(),
            option,
            backend: std::sync::Arc::new(NalgebraBackend::default()),
        };
        let msg = holo.into_datagram_lightweight(None).unwrap();
        match msg.datagram {
            Some(raw_datagram::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Naive(g)),
                ..
            })) => {
                let holo2 = autd3_gain_holo::Naive::from_msg(g).unwrap();
                assert_eq!(option, holo2.option);
                foci.iter().zip(holo2.foci.iter()).for_each(|(f1, f2)| {
                    assert_eq!(f1, f2);
                });
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
