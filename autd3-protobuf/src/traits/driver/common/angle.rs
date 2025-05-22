use crate::{AUTDProtoBufError, pb::*, traits::FromMessage};

impl From<autd3_core::common::Angle> for Angle {
    fn from(value: autd3_core::common::Angle) -> Self {
        Self {
            rad: value.radian(),
        }
    }
}

impl FromMessage<Angle> for autd3_core::common::Angle {
    fn from_msg(msg: Angle) -> Result<Self, AUTDProtoBufError> {
        Ok(msg.rad * autd3_core::common::rad)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3_core::common::{Angle, rad};
    use rand::Rng;

    #[test]
    fn angle() {
        let mut rng = rand::rng();
        let v = rng.random::<f32>() * rad;
        let msg = v.into();
        let v2 = Angle::from_msg(msg).unwrap();
        approx::assert_abs_diff_eq!(v.radian(), v2.radian());
    }
}
