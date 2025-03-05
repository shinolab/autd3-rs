use std::num::NonZeroU8;

use crate::{
    AUTDProtoBufError,
    pb::*,
    to_holo,
    traits::{FromMessage, ToMessage, driver::datagram::gain::IntoLightweightGain},
};
use autd3_core::acoustics::directivity::Sphere;

impl ToMessage for autd3_gain_holo::GreedyOption<Sphere> {
    type Message = GreedyOption;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(Self::Message {
            phase_div: Some(self.phase_div.get() as _),
            constraint: Some(self.constraint.to_msg(None)?),
        })
    }
}

impl FromMessage<GreedyOption> for autd3_gain_holo::GreedyOption<Sphere> {
    fn from_msg(msg: GreedyOption) -> Result<Self, AUTDProtoBufError> {
        let default = autd3_gain_holo::GreedyOption::<Sphere>::default();
        Ok(Self {
            phase_div: msg
                .phase_div
                .map(u8::try_from)
                .transpose()?
                .map(NonZeroU8::try_from)
                .transpose()?
                .unwrap_or(default.phase_div),
            constraint: msg
                .constraint
                .map(autd3_gain_holo::EmissionConstraint::from_msg)
                .transpose()?
                .unwrap_or(default.constraint),
            __phantom: std::marker::PhantomData,
        })
    }
}

impl IntoLightweightGain for autd3_gain_holo::Greedy<Sphere> {
    fn into_lightweight(&self) -> Gain {
        Gain {
            gain: Some(gain::Gain::Greedy(Greedy {
                holo: to_holo!(self),
                option: Some(self.option.to_msg(None).unwrap()),
            })),
        }
    }
}

impl FromMessage<Greedy> for autd3_gain_holo::Greedy<Sphere> {
    fn from_msg(msg: Greedy) -> Result<Self, AUTDProtoBufError> {
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
            option: autd3_gain_holo::GreedyOption::from_msg(
                msg.option.ok_or(AUTDProtoBufError::DataParseError)?,
            )?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3_core::geometry::Point3;
    use rand::Rng;

    #[test]
    fn test_holo_greedy() {
        let mut rng = rand::rng();

        let holo = autd3_gain_holo::Greedy {
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
            option: autd3_gain_holo::GreedyOption {
                phase_div: NonZeroU8::new(rng.random()).unwrap(),
                ..Default::default()
            },
        };
        let msg = holo.to_msg(None).unwrap();
        match msg.datagram {
            Some(datagram::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Greedy(g)),
                ..
            })) => {
                let holo2 = autd3_gain_holo::Greedy::from_msg(g).unwrap();
                assert_eq!(holo.option.phase_div, holo2.option.phase_div);
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
