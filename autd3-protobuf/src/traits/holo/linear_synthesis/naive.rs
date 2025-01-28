use autd3_gain_holo::NalgebraBackend;

use crate::{
    pb::*,
    to_holo,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

impl ToMessage
    for autd3_gain_holo::Naive<
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
                gain: Some(gain::Gain::Naive(Naive {
                    holo: to_holo!(self),
                    constraint: Some(self.option.constraint.to_msg(None)?),
                })),
            })),
        })
    }
}

impl FromMessage<Naive>
    for autd3_gain_holo::Naive<
        autd3_core::acoustics::directivity::Sphere,
        NalgebraBackend<autd3_core::acoustics::directivity::Sphere>,
    >
{
    fn from_msg(msg: &Naive) -> Result<Self, AUTDProtoBufError> {
        Ok(Self {
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
            option: autd3_gain_holo::NaiveOption {
                constraint:
                    msg.constraint
                        .as_ref()
                        .map(autd3_gain_holo::EmissionConstraint::from_msg)
                        .transpose()?
                        .unwrap_or(
                            autd3_gain_holo::NaiveOption::<
                                autd3_core::acoustics::directivity::Sphere,
                            >::default()
                            .constraint,
                        ),
                ..Default::default()
            },
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
    fn test_holo_naive() {
        let mut rng = rand::rng();

        let holo = autd3_gain_holo::Naive {
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
            option: autd3_gain_holo::NaiveOption {
                ..Default::default()
            },
            backend: std::sync::Arc::new(NalgebraBackend::default()),
        };
        let msg = holo.to_msg(None).unwrap();
        match msg.datagram {
            Some(datagram::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Naive(g)),
                ..
            })) => {
                let holo2 = autd3_gain_holo::Naive::from_msg(&g).unwrap();
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
