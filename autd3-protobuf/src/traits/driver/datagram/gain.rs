use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3_driver::datagram::ChangeGainSegment {
    type Message = ChangeGainSegment;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            segment: self.segment() as _,
        }
    }
}

impl FromMessage<ChangeGainSegment> for autd3_driver::datagram::ChangeGainSegment {
    fn from_msg(msg: &ChangeGainSegment) -> Option<Self> {
        Some(autd3_driver::datagram::ChangeGainSegment::new(
            autd3_driver::firmware::fpga::Segment::from(Segment::try_from(msg.segment).ok()?),
        ))
    }
}
