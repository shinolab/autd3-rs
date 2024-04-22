use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3_driver::operation::stm::ControlPoint {
    type Message = focus_stm::ControlPoint;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            pos: Some(self.point().to_msg(None)),
            intensity: Some(self.intensity().to_msg(None)),
        }
    }
}

impl FromMessage<focus_stm::ControlPoint> for autd3_driver::operation::stm::ControlPoint {
    #[allow(clippy::unnecessary_cast)]
    fn from_msg(msg: &focus_stm::ControlPoint) -> Option<Self> {
        Some(
            autd3_driver::operation::stm::ControlPoint::new(
                msg.pos
                    .as_ref()
                    .map(autd3_driver::geometry::Vector3::from_msg)??,
            )
            .with_intensity(
                msg.intensity
                    .as_ref()
                    .map(autd3_driver::fpga::EmitIntensity::from_msg)??,
            ),
        )
    }
}
