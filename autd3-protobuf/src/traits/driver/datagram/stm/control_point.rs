use crate::{AUTDProtoBufError, pb::*, traits::FromMessage};

impl From<autd3_driver::datagram::ControlPoint> for ControlPoint {
    fn from(value: autd3_driver::datagram::ControlPoint) -> Self {
        Self {
            pos: Some(value.point.into()),
            offset: Some(value.phase_offset.into()),
        }
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

impl<const N: usize> From<autd3_driver::datagram::ControlPoints<N>> for ControlPoints {
    fn from(value: autd3_driver::datagram::ControlPoints<N>) -> Self {
        Self {
            points: value.into_iter().map(|p| p.into()).collect(),
            intensity: Some(value.intensity.into()),
        }
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
