use crate::{
    AUTDProtoBufError,
    pb::*,
    traits::{FromMessage, ToMessage},
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
    fn from_msg(msg: ControlPoint) -> Result<Self, AUTDProtoBufError> {
        Ok(Self {
            point: autd3_core::geometry::Point3::from_msg(
                msg.pos.ok_or(AUTDProtoBufError::DataParseError)?,
            )?,
            phase_offset: msg
                .offset
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

impl<const N: usize> FromMessage<ControlPoints> for autd3_driver::datagram::ControlPoints<N> {
    fn from_msg(msg: ControlPoints) -> Result<Self, AUTDProtoBufError> {
        Ok(Self {
            points: msg
                .points
                .into_iter()
                .map(autd3_driver::datagram::ControlPoint::from_msg)
                .collect::<Result<Vec<_>, _>>()?
                .as_slice()
                .try_into()
                .map_err(|_| AUTDProtoBufError::DataParseError)?,
            intensity: msg
                .intensity
                .map(autd3_driver::firmware::fpga::EmitIntensity::from_msg)
                .transpose()?
                .unwrap_or(autd3_driver::datagram::ControlPoints::<N>::default().intensity),
        })
    }
}
