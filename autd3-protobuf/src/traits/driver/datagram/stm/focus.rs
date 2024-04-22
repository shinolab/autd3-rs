use autd3_driver::derive::SamplingConfiguration;

use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3_driver::datagram::FocusSTM {
    type Message = FocusStm;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            freq_div: self.sampling_config().unwrap().frequency_division(),
            loop_behavior: Some(self.loop_behavior().to_msg(None)),
            segment: Segment::S0 as _,
            update_segment: true,
            points: self.foci().iter().map(|p| p.to_msg(None)).collect(),
        }
    }
}

impl ToMessage for autd3_driver::datagram::DatagramWithSegment<autd3_driver::datagram::FocusSTM> {
    type Message = FocusStm;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            freq_div: self.sampling_config().unwrap().frequency_division(),
            loop_behavior: Some(self.loop_behavior().to_msg(None)),
            segment: self.segment() as _,
            update_segment: self.update_segment(),
            points: self.foci().iter().map(|p| p.to_msg(None)).collect(),
        }
    }
}

impl FromMessage<FocusStm> for autd3_driver::datagram::FocusSTM {
    #[allow(clippy::unnecessary_cast)]
    fn from_msg(msg: &FocusStm) -> Option<Self> {
        autd3_driver::datagram::FocusSTM::from_sampling_config(
            SamplingConfiguration::from_frequency_division(msg.freq_div).ok()?,
        )
        .with_loop_behavior(autd3_driver::fpga::LoopBehavior::from_msg(
            msg.loop_behavior.as_ref()?,
        )?)
        .add_foci_from_iter(
            msg.points
                .iter()
                .filter_map(autd3_driver::operation::stm::ControlPoint::from_msg),
        )
        .ok()
    }
}

impl ToMessage for autd3_driver::datagram::ChangeFocusSTMSegment {
    type Message = ChangeFocusStmSegment;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            segment: self.segment() as _,
        }
    }
}

impl FromMessage<ChangeFocusStmSegment> for autd3_driver::datagram::ChangeFocusSTMSegment {
    fn from_msg(msg: &ChangeFocusStmSegment) -> Option<Self> {
        Some(autd3_driver::datagram::ChangeFocusSTMSegment::new(
            autd3_driver::fpga::Segment::from(Segment::try_from(msg.segment).ok()?),
        ))
    }
}
