use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

impl ToMessage for autd3_driver::defined::ControlPoint {
    type Message = ControlPoint;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            pos: Some(self.point().to_msg(None)),
            offset: Some(self.phase_offset().to_msg(None)),
        }
    }
}

impl FromMessage<ControlPoint> for autd3_driver::defined::ControlPoint {
    fn from_msg(msg: &ControlPoint) -> Result<Self, AUTDProtoBufError> {
        let mut p = autd3_driver::defined::ControlPoint::new(
            autd3_driver::geometry::Vector3::from_msg(&msg.pos)?,
        );
        if let Some(offset) = msg.offset.as_ref() {
            p = p.with_phase_offset(autd3_driver::firmware::fpga::Phase::from_msg(offset)?);
        }
        Ok(p)
    }
}

impl<const N: usize> ToMessage for autd3_driver::defined::ControlPoints<N> {
    type Message = ControlPoints;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            points: self.iter().map(|p| p.to_msg(None)).collect(),
            intensity: Some(self.intensity().to_msg(None)),
        }
    }
}
