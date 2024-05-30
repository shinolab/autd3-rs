use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3_driver::defined::Angle {
    type Message = Angle;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            rad: self.radian() as _,
        }
    }
}

impl FromMessage<Angle> for autd3_driver::defined::Angle {
    fn from_msg(msg: &Angle) -> Option<Self> {
        Some(msg.rad as f32 * autd3_driver::defined::rad)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3_driver::defined::{rad, Angle};
    use rand::Rng;

    #[test]
    fn angle() {
        let mut rng = rand::thread_rng();
        let v = rng.gen::<f32>() * rad;
        let msg = v.to_msg(None);
        let v2 = Angle::from_msg(&msg).unwrap();
        assert_approx_eq::assert_approx_eq!(v.radian(), v2.radian());
    }
}
