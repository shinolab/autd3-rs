use crate::{pb::*, traits::*};

impl ToMessage for autd3_driver::datagram::ChangeModulationSegment {
    type Message = ChangeModulationSegment;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            segment: self.segment() as _,
            transition_mode: self.transition_mode().mode() as _,
            transition_value: self.transition_mode().value(),
        }
    }
}

impl FromMessage<ChangeModulationSegment> for autd3_driver::datagram::ChangeModulationSegment {
    fn from_msg(msg: &ChangeModulationSegment) -> Option<Self> {
        Some(autd3_driver::datagram::ChangeModulationSegment::new(
            autd3_driver::firmware::fpga::Segment::from(Segment::try_from(msg.segment).ok()?),
            to_transition_mode(Some(msg.transition_mode), Some(msg.transition_value)).unwrap(),
        ))
    }
}
