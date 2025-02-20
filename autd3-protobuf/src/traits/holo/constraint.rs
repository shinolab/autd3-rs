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
                variant: Some(emission_constraint::Variant::Normalize(
                    emission_constraint::Normalize {},
                )),
            },
            autd3_gain_holo::EmissionConstraint::Multiply(value) => Self::Message {
                variant: Some(emission_constraint::Variant::Multiply(
                    emission_constraint::Multiply { value: *value as _ },
                )),
            },
            autd3_gain_holo::EmissionConstraint::Uniform(value) => Self::Message {
                variant: Some(emission_constraint::Variant::Uniform(
                    emission_constraint::Uniform {
                        value: Some(value.to_msg(None)?),
                    },
                )),
            },
            autd3_gain_holo::EmissionConstraint::Clamp(min, max) => Self::Message {
                variant: Some(emission_constraint::Variant::Clamp(
                    emission_constraint::Clamp {
                        min: Some(min.to_msg(None)?),
                        max: Some(max.to_msg(None)?),
                    },
                )),
            },
        })
    }
}

impl FromMessage<EmissionConstraint> for autd3_gain_holo::EmissionConstraint {
    fn from_msg(msg: EmissionConstraint) -> Result<Self, AUTDProtoBufError> {
        match msg.variant.ok_or(AUTDProtoBufError::DataParseError)? {
            emission_constraint::Variant::Normalize(_) => {
                Ok(autd3_gain_holo::EmissionConstraint::Normalize)
            }
            emission_constraint::Variant::Multiply(v) => {
                Ok(autd3_gain_holo::EmissionConstraint::Multiply(v.value))
            }
            emission_constraint::Variant::Uniform(v) => {
                Ok(autd3_gain_holo::EmissionConstraint::Uniform(
                    EmitIntensity::from_msg(v.value.ok_or(AUTDProtoBufError::DataParseError)?)?,
                ))
            }
            emission_constraint::Variant::Clamp(v) => {
                Ok(autd3_gain_holo::EmissionConstraint::Clamp(
                    EmitIntensity::from_msg(v.min.ok_or(AUTDProtoBufError::DataParseError)?)?,
                    EmitIntensity::from_msg(v.max.ok_or(AUTDProtoBufError::DataParseError)?)?,
                ))
            }
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
        let v2 = autd3_gain_holo::EmissionConstraint::from_msg(msg).unwrap();
        assert_eq!(v, v2);
    }

    #[test]
    fn test_emission_constraint_uniform() {
        let mut rng = rand::rng();
        let v = autd3_gain_holo::EmissionConstraint::Uniform(EmitIntensity(rng.random()));
        let msg = v.to_msg(None).unwrap();
        let v2 = autd3_gain_holo::EmissionConstraint::from_msg(msg).unwrap();
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
        let v2 = autd3_gain_holo::EmissionConstraint::from_msg(msg).unwrap();
        assert_eq!(v, v2);
    }
}
