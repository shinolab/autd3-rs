use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};
use autd3_driver::firmware::fpga::EmitIntensity;

impl ToMessage for autd3_gain_holo::EmissionConstraint {
    type Message = EmissionConstraint;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(match self {
            autd3_gain_holo::EmissionConstraint::Normalize => Self::Message {
                constraint: Some(emission_constraint::Constraint::Normalize(
                    NormalizeConstraint {},
                )),
            },
            autd3_gain_holo::EmissionConstraint::Multiply(value) => Self::Message {
                constraint: Some(emission_constraint::Constraint::Multiply(
                    MultiplyConstraint { value: *value as _ },
                )),
            },
            autd3_gain_holo::EmissionConstraint::Uniform(value) => Self::Message {
                constraint: Some(emission_constraint::Constraint::Uniform(
                    UniformConstraint {
                        value: Some(value.to_msg(None)?),
                    },
                )),
            },
            autd3_gain_holo::EmissionConstraint::Clamp(min, max) => Self::Message {
                constraint: Some(emission_constraint::Constraint::Clamp(ClampConstraint {
                    min: Some(min.to_msg(None)?),
                    max: Some(max.to_msg(None)?),
                })),
            },
            _ => unimplemented!(),
        })
    }
}

impl FromMessage<EmissionConstraint> for autd3_gain_holo::EmissionConstraint {
    fn from_msg(msg: &EmissionConstraint) -> Result<Self, AUTDProtoBufError> {
        match msg.constraint {
            Some(emission_constraint::Constraint::Normalize(_)) => {
                Ok(autd3_gain_holo::EmissionConstraint::Normalize)
            }
            Some(emission_constraint::Constraint::Multiply(ref v)) => {
                Ok(autd3_gain_holo::EmissionConstraint::Multiply(v.value as _))
            }
            Some(emission_constraint::Constraint::Uniform(ref v)) => Ok(
                autd3_gain_holo::EmissionConstraint::Uniform(EmitIntensity::from_msg(
                    v.value.as_ref().ok_or(AUTDProtoBufError::DataParseError)?,
                )?),
            ),
            Some(emission_constraint::Constraint::Clamp(ref v)) => {
                Ok(autd3_gain_holo::EmissionConstraint::Clamp(
                    EmitIntensity::from_msg(
                        v.min.as_ref().ok_or(AUTDProtoBufError::DataParseError)?,
                    )?,
                    EmitIntensity::from_msg(
                        v.max.as_ref().ok_or(AUTDProtoBufError::DataParseError)?,
                    )?,
                ))
            }
            None => Err(AUTDProtoBufError::DataParseError),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3_driver::firmware::fpga::EmitIntensity;
    use rand::Rng;

    #[test]
    fn test_emission_constraint_normalize() {
        let v = autd3_gain_holo::EmissionConstraint::Normalize;
        let msg = v.to_msg(None).unwrap();
        let v2 = autd3_gain_holo::EmissionConstraint::from_msg(&msg).unwrap();
        assert_eq!(v, v2);
    }

    #[test]
    fn test_emission_constraint_uniform() {
        let mut rng = rand::rng();
        let v = autd3_gain_holo::EmissionConstraint::Uniform(EmitIntensity(rng.random()));
        let msg = v.to_msg(None).unwrap();
        let v2 = autd3_gain_holo::EmissionConstraint::from_msg(&msg).unwrap();
        assert_eq!(v, v2);
    }

    #[test]
    fn test_emission_constraint_clamp() {
        let mut rng = rand::rng();
        let v = autd3_gain_holo::EmissionConstraint::Clamp(
            EmitIntensity(rng.random()),
            EmitIntensity(rng.random()),
        );
        let msg = v.to_msg(None).unwrap();
        let v2 = autd3_gain_holo::EmissionConstraint::from_msg(&msg).unwrap();
        assert_eq!(v, v2);
    }
}
