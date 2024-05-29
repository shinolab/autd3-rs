use crate::{
    pb::*,
    traits::{to_transition_mode, FromMessage, ToMessage},
};

impl ToMessage for autd3_driver::datagram::SwapSegment<autd3_driver::datagram::segment::Gain> {
    type Message = SwapSegmentGain;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            segment: self.segment() as _,
        }
    }
}

impl FromMessage<SwapSegmentGain>
    for autd3_driver::datagram::SwapSegment<autd3_driver::datagram::segment::Gain>
{
    fn from_msg(msg: &SwapSegmentGain) -> Option<Self> {
        Some(autd3_driver::datagram::SwapSegment::Gain(
            autd3_driver::firmware::fpga::Segment::from(Segment::try_from(msg.segment).ok()?),
        ))
    }
}

impl ToMessage
    for autd3_driver::datagram::SwapSegment<autd3_driver::datagram::segment::Modulation>
{
    type Message = SwapSegmentModulation;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            segment: self.segment() as _,
            transition_mode: self.transition_mode().mode() as _,
            transition_value: self.transition_mode().value(),
        }
    }
}

impl FromMessage<SwapSegmentModulation>
    for autd3_driver::datagram::SwapSegment<autd3_driver::datagram::segment::Modulation>
{
    fn from_msg(msg: &SwapSegmentModulation) -> Option<Self> {
        Some(autd3_driver::datagram::SwapSegment::Modulation(
            autd3_driver::firmware::fpga::Segment::from(Segment::try_from(msg.segment).ok()?),
            to_transition_mode(Some(msg.transition_mode), Some(msg.transition_value)).unwrap(),
        ))
    }
}

impl ToMessage for autd3_driver::datagram::SwapSegment<autd3_driver::datagram::segment::FocusSTM> {
    type Message = SwapSegmentFocusStm;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            segment: self.segment() as _,
            transition_mode: self.transition_mode().mode() as _,
            transition_value: self.transition_mode().value(),
        }
    }
}

impl FromMessage<SwapSegmentFocusStm>
    for autd3_driver::datagram::SwapSegment<autd3_driver::datagram::segment::FocusSTM>
{
    fn from_msg(msg: &SwapSegmentFocusStm) -> Option<Self> {
        Some(autd3_driver::datagram::SwapSegment::FocusSTM(
            autd3_driver::firmware::fpga::Segment::from(Segment::try_from(msg.segment).ok()?),
            to_transition_mode(Some(msg.transition_mode), Some(msg.transition_value)).unwrap(),
        ))
    }
}

impl ToMessage for autd3_driver::datagram::SwapSegment<autd3_driver::datagram::segment::GainSTM> {
    type Message = SwapSegmentGainStm;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            segment: self.segment() as _,
            transition_mode: self.transition_mode().mode() as _,
            transition_value: self.transition_mode().value(),
        }
    }
}

impl FromMessage<SwapSegmentGainStm>
    for autd3_driver::datagram::SwapSegment<autd3_driver::datagram::segment::GainSTM>
{
    fn from_msg(msg: &SwapSegmentGainStm) -> Option<Self> {
        Some(autd3_driver::datagram::SwapSegment::GainSTM(
            autd3_driver::firmware::fpga::Segment::from(Segment::try_from(msg.segment).ok()?),
            to_transition_mode(Some(msg.transition_mode), Some(msg.transition_value)).unwrap(),
        ))
    }
}
