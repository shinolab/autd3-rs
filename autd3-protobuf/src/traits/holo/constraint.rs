use autd3_driver::derive::EmitIntensity;

use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3_gain_holo::EmissionConstraint {
    type Message = EmissionConstraint;

    #[allow(clippy::unnecessary_cast)]
    fn to_msg(&self) -> Self::Message {
        match self {
            autd3_gain_holo::EmissionConstraint::DontCare => Self::Message {
                constraint: Some(emission_constraint::Constraint::DontCare(
                    DontCareConstraint {},
                )),
            },
            autd3_gain_holo::EmissionConstraint::Normalize => Self::Message {
                constraint: Some(emission_constraint::Constraint::Normalize(
                    NormalizeConstraint {},
                )),
            },
            autd3_gain_holo::EmissionConstraint::Uniform(value) => Self::Message {
                constraint: Some(emission_constraint::Constraint::Uniform(
                    UniformConstraint {
                        value: Some(value.to_msg()),
                    },
                )),
            },
            autd3_gain_holo::EmissionConstraint::Clamp(min, max) => Self::Message {
                constraint: Some(emission_constraint::Constraint::Clamp(ClampConstraint {
                    min: Some(min.to_msg()),
                    max: Some(max.to_msg()),
                })),
            },
        }
    }
}

impl FromMessage<EmissionConstraint> for autd3_gain_holo::EmissionConstraint {
    fn from_msg(msg: &EmissionConstraint) -> Option<Self> {
        match msg.constraint {
            Some(emission_constraint::Constraint::DontCare(_)) => {
                Some(autd3_gain_holo::EmissionConstraint::DontCare)
            }
            Some(emission_constraint::Constraint::Normalize(_)) => {
                Some(autd3_gain_holo::EmissionConstraint::Normalize)
            }
            Some(emission_constraint::Constraint::Uniform(ref v)) => {
                Some(autd3_gain_holo::EmissionConstraint::Uniform(
                    EmitIntensity::from_msg(v.value.as_ref()?)?,
                ))
            }
            Some(emission_constraint::Constraint::Clamp(ref v)) => {
                Some(autd3_gain_holo::EmissionConstraint::Clamp(
                    EmitIntensity::from_msg(v.min.as_ref()?)?,
                    EmitIntensity::from_msg(v.max.as_ref()?)?,
                ))
            }
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3_driver::derive::EmitIntensity;
    use rand::Rng;

    #[test]
    fn test_emission_constraint_dont_care() {
        let v = autd3_gain_holo::EmissionConstraint::DontCare;
        let msg = v.to_msg();
        let v2 = autd3_gain_holo::EmissionConstraint::from_msg(&msg).unwrap();
        assert_eq!(v, v2);
    }

    #[test]
    fn test_emission_constraint_normalize() {
        let v = autd3_gain_holo::EmissionConstraint::Normalize;
        let msg = v.to_msg();
        let v2 = autd3_gain_holo::EmissionConstraint::from_msg(&msg).unwrap();
        assert_eq!(v, v2);
    }

    #[test]
    fn test_emission_constraint_uniform() {
        let mut rng = rand::thread_rng();
        let v = autd3_gain_holo::EmissionConstraint::Uniform(EmitIntensity::new(rng.gen()));
        let msg = v.to_msg();
        let v2 = autd3_gain_holo::EmissionConstraint::from_msg(&msg).unwrap();
        assert_eq!(v, v2);
    }

    #[test]
    fn test_emission_constraint_clamp() {
        let mut rng = rand::thread_rng();
        let v = autd3_gain_holo::EmissionConstraint::Clamp(
            EmitIntensity::new(rng.gen()),
            EmitIntensity::new(rng.gen()),
        );
        let msg = v.to_msg();
        let v2 = autd3_gain_holo::EmissionConstraint::from_msg(&msg).unwrap();
        assert_eq!(v, v2);
    }
}
