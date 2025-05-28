use std::num::NonZeroU8;

use crate::{
    AUTDProtoBufError,
    pb::*,
    to_holo,
    traits::{FromMessage, driver::datagram::gain::IntoLightweightGain},
};
use autd3_core::acoustics::directivity::Sphere;
use autd3_gain_holo::AbsGreedyObjectiveFn;

impl From<autd3_gain_holo::GreedyOption<Sphere, AbsGreedyObjectiveFn>> for GreedyOption {
    fn from(value: autd3_gain_holo::GreedyOption<Sphere, AbsGreedyObjectiveFn>) -> Self {
        Self {
            phase_quantization_levels: Some(value.phase_quantization_levels.get() as _),
            constraint: Some(value.constraint.into()),
        }
    }
}

impl FromMessage<GreedyOption> for autd3_gain_holo::GreedyOption<Sphere, AbsGreedyObjectiveFn> {
    fn from_msg(msg: GreedyOption) -> Result<Self, AUTDProtoBufError> {
        let default = autd3_gain_holo::GreedyOption::<Sphere, AbsGreedyObjectiveFn>::default();
        Ok(Self {
            phase_quantization_levels: msg
                .phase_quantization_levels
                .map(u8::try_from)
                .transpose()?
                .map(NonZeroU8::try_from)
                .transpose()?
                .unwrap_or(default.phase_quantization_levels),
            constraint: msg
                .constraint
                .map(autd3_gain_holo::EmissionConstraint::from_msg)
                .transpose()?
                .unwrap_or(default.constraint),
            objective_func: AbsGreedyObjectiveFn,
            __phantom: std::marker::PhantomData,
        })
    }
}

impl IntoLightweightGain for autd3_gain_holo::Greedy<Sphere, AbsGreedyObjectiveFn> {
    fn into_lightweight(self) -> Gain {
        Gain {
            gain: Some(gain::Gain::Greedy(Greedy {
                holo: to_holo!(self),
                option: Some(self.option.into()),
            })),
        }
    }
}

impl FromMessage<Greedy> for autd3_gain_holo::Greedy<Sphere, AbsGreedyObjectiveFn> {
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
    use crate::DatagramLightweight;

    use super::*;
    use autd3_core::geometry::Point3;
    use rand::Rng;

    #[test]
    fn test_holo_greedy() {
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
        let option = autd3_gain_holo::GreedyOption {
            phase_quantization_levels: NonZeroU8::new(rng.random()).unwrap_or(NonZeroU8::MIN),
            ..Default::default()
        };
        let holo = autd3_gain_holo::Greedy {
            foci: foci.clone(),
            option,
        };
        let msg = holo.into_datagram_lightweight(None).unwrap();
        match msg.datagram {
            Some(raw_datagram::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Greedy(g)),
                ..
            })) => {
                let holo2 = autd3_gain_holo::Greedy::from_msg(g).unwrap();
                assert_eq!(option, holo2.option);
                foci.iter().zip(holo2.foci.iter()).for_each(|(f1, f2)| {
                    assert_eq!(f1, f2);
                });
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
