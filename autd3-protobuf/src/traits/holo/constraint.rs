use crate::{AUTDProtoBufError, pb::*, traits::FromMessage};
use autd3_driver::firmware::fpga::EmitIntensity;

impl From<autd3_gain_holo::EmissionConstraint> for EmissionConstraint {
    fn from(value: autd3_gain_holo::EmissionConstraint) -> Self {
        match value {
            autd3_gain_holo::EmissionConstraint::Normalize => Self {
                variant: Some(emission_constraint::Variant::Normalize(
                    emission_constraint::Normalize {},
                )),
            },
            autd3_gain_holo::EmissionConstraint::Multiply(value) => Self {
                variant: Some(emission_constraint::Variant::Multiply(
                    emission_constraint::Multiply { value: value as _ },
                )),
            },
            autd3_gain_holo::EmissionConstraint::Uniform(value) => Self {
                variant: Some(emission_constraint::Variant::Uniform(
                    emission_constraint::Uniform {
                        value: Some(value.into()),
                    },
                )),
            },
            autd3_gain_holo::EmissionConstraint::Clamp(min, max) => Self {
                variant: Some(emission_constraint::Variant::Clamp(
                    emission_constraint::Clamp {
                        min: Some(min.into()),
                        max: Some(max.into()),
                    },
                )),
            },
        }
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
        let msg = v.into();
        let v2 = autd3_gain_holo::EmissionConstraint::from_msg(msg).unwrap();
        assert_eq!(v, v2);
    }

    #[test]
    fn test_emission_constraint_uniform() {
        let mut rng = rand::rng();
        let v = autd3_gain_holo::EmissionConstraint::Uniform(EmitIntensity(rng.random()));
        let msg = v.into();
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
        let msg = v.into();
        let v2 = autd3_gain_holo::EmissionConstraint::from_msg(msg).unwrap();
        assert_eq!(v, v2);
    }
}
