use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3_driver::datagram::ChangeModulationSegment {
    type Message = ChangeModulationSegment;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            segment: self.segment() as _,
        }
    }
}

impl FromMessage<ChangeModulationSegment> for autd3_driver::datagram::ChangeModulationSegment {
    fn from_msg(msg: &ChangeModulationSegment) -> Option<Self> {
        Some(autd3_driver::datagram::ChangeModulationSegment::new(
            autd3_driver::fpga::Segment::from(Segment::try_from(msg.segment).ok()?),
        ))
    }
}
