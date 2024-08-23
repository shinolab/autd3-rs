use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

impl ToMessage for autd3_driver::defined::Angle {
    type Message = Angle;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            rad: self.radian() as _,
        }
    }
}

impl FromMessage<Option<Angle>> for autd3_driver::defined::Angle {
    fn from_msg(msg: &Option<Angle>) -> Result<Self, AUTDProtoBufError> {
        match msg {
            None => Err(AUTDProtoBufError::DataParseError),
            Some(msg) => Ok(msg.rad * autd3_driver::defined::rad),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3_driver::defined::{rad, Angle};
    use rand::Rng;

    #[test]
    #[cfg_attr(miri, ignore)]
    fn angle() {
        let mut rng = rand::thread_rng();
        let v = rng.gen::<f32>() * rad;
        let msg = v.to_msg(None);
        let v2 = Angle::from_msg(&Some(msg)).unwrap();
        approx::assert_abs_diff_eq!(v.radian(), v2.radian());
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn parse_error() {
        assert!(Angle::from_msg(&None).is_err());
    }
}
