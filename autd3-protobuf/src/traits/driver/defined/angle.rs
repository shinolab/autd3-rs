use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

impl ToMessage for autd3_core::defined::Angle {
    type Message = Angle;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(Self::Message {
            rad: self.radian() as _,
        })
    }
}

impl FromMessage<Option<Angle>> for autd3_core::defined::Angle {
    fn from_msg(msg: &Option<Angle>) -> Result<Self, AUTDProtoBufError> {
        msg.map(|msg| msg.rad * autd3_core::defined::rad)
            .ok_or(AUTDProtoBufError::DataParseError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3_core::defined::{rad, Angle};
    use rand::Rng;

    #[test]
    fn angle() {
        let mut rng = rand::rng();
        let v = rng.random::<f32>() * rad;
        let msg = v.to_msg(None).unwrap();
        let v2 = Angle::from_msg(&Some(msg)).unwrap();
        approx::assert_abs_diff_eq!(v.radian(), v2.radian());
    }

    #[test]
    fn parse_error() {
        assert!(Angle::from_msg(&None).is_err());
    }
}
