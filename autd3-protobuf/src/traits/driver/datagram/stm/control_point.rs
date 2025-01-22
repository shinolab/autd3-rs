use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

impl ToMessage for autd3_driver::datagram::ControlPoint {
    type Message = ControlPoint;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(Self::Message {
            pos: Some(self.point.to_msg(None)?),
            offset: Some(self.phase_offset.to_msg(None)?),
        })
    }
}

impl FromMessage<ControlPoint> for autd3_driver::datagram::ControlPoint {
    fn from_msg(msg: &ControlPoint) -> Result<Self, AUTDProtoBufError> {
        Ok(autd3_driver::datagram::ControlPoint {
            point: autd3_core::geometry::Point3::from_msg(&msg.pos)?,
            phase_offset: msg
                .offset
                .as_ref()
                .map(autd3_driver::firmware::fpga::Phase::from_msg)
                .transpose()?
                .unwrap_or(autd3_driver::datagram::ControlPoint::default().phase_offset),
        })
    }
}

impl<const N: usize> ToMessage for autd3_driver::datagram::ControlPoints<N> {
    type Message = ControlPoints;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(Self::Message {
            points: self
                .iter()
                .map(|p| p.to_msg(None))
                .collect::<Result<_, _>>()?,
            intensity: Some(self.intensity.to_msg(None)?),
        })
    }
}
