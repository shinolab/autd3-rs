use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3_driver::defined::ControlPoint {
    type Message = ControlPoint;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            pos: Some(self.point().to_msg(None)),
            offset: Some(self.offset().to_msg(None)),
        }
    }
}

impl FromMessage<ControlPoint> for autd3_driver::defined::ControlPoint {
    #[allow(clippy::unnecessary_cast)]
    fn from_msg(msg: &ControlPoint) -> Option<Self> {
        Some(
            autd3_driver::defined::ControlPoint::new(
                msg.pos
                    .as_ref()
                    .map(autd3_driver::geometry::Vector3::from_msg)??,
            )
            .with_offset(
                msg.offset
                    .as_ref()
                    .map(autd3_driver::firmware::fpga::Phase::from_msg)??,
            ),
        )
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
