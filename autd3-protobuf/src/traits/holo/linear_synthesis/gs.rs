use std::num::NonZeroUsize;

use autd3_gain_holo::NalgebraBackend;

use crate::{
    pb::*,
    to_holo,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

impl ToMessage
    for autd3_gain_holo::GS<
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
                gain: Some(gain::Gain::Gs(Gs {
                    holo: to_holo!(self),
                    repeat: Some(self.option.repeat.get() as _),
                    constraint: Some(self.option.constraint.to_msg(None)?),
                })),
            })),
        })
    }
}

impl FromMessage<Gs>
    for autd3_gain_holo::GS<
        autd3_core::acoustics::directivity::Sphere,
        NalgebraBackend<autd3_core::acoustics::directivity::Sphere>,
    >
{
    fn from_msg(msg: &Gs) -> Result<Self, AUTDProtoBufError> {
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
            option:
                autd3_gain_holo::GSOption {
                    repeat:
                        msg.repeat
                            .map(usize::try_from)
                            .transpose()?
                            .map(|x| NonZeroUsize::new(x).ok_or(AUTDProtoBufError::DataParseError))
                            .transpose()?
                            .unwrap_or(
                                autd3_gain_holo::GSOption::<
                                    autd3_core::acoustics::directivity::Sphere,
                                >::default()
                                .repeat,
                            ),
                    constraint:
                        msg.constraint
                            .as_ref()
                            .map(autd3_gain_holo::EmissionConstraint::from_msg)
                            .transpose()?
                            .unwrap_or(
                                autd3_gain_holo::GSOption::<
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
    fn test_holo_gs() {
        let mut rng = rand::thread_rng();

        let holo = autd3_gain_holo::GS {
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
            option: autd3_gain_holo::GSOption {
                repeat: NonZeroUsize::new(rng.gen()).unwrap(),
                ..Default::default()
            },
            backend: std::sync::Arc::new(NalgebraBackend::default()),
        };
        let msg = holo.to_msg(None).unwrap();
        match msg.datagram {
            Some(datagram::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Gs(g)),
                ..
            })) => {
                let holo2 = autd3_gain_holo::GS::from_msg(&g).unwrap();
                assert_eq!(holo.option.repeat, holo2.option.repeat);
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
