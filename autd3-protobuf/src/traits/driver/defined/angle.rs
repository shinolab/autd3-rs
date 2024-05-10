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
        Some(msg.rad as f64 * autd3_driver::defined::rad)
    }
}
