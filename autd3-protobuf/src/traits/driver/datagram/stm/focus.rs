use autd3_driver::derive::SamplingConfig;

use crate::{pb::*, traits::*};

impl ToMessage for autd3_driver::datagram::FocusSTM {
    type Message = FocusStm;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            freq_div: self.sampling_config().unwrap().division(),
            loop_behavior: Some(self.loop_behavior().to_msg(None)),
            segment: Segment::S0 as _,
            transition_mode: Some(TransitionMode::SyncIdx.into()),
            transition_value: Some(0),
            points: self.foci().iter().map(|p| p.to_msg(None)).collect(),
        }
    }
}

impl ToMessage for autd3_driver::datagram::DatagramWithSegment<autd3_driver::datagram::FocusSTM> {
    type Message = FocusStm;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            freq_div: self.sampling_config().unwrap().division(),
            loop_behavior: Some(self.loop_behavior().to_msg(None)),
            segment: self.segment() as _,
            transition_mode: self.transition_mode().map(|m| m.mode() as _),
            transition_value: self.transition_mode().map(|m| m.value()),
            points: self.foci().iter().map(|p| p.to_msg(None)).collect(),
        }
    }
}

impl FromMessage<FocusStm> for autd3_driver::datagram::FocusSTM {
    #[allow(clippy::unnecessary_cast)]
    fn from_msg(msg: &FocusStm) -> Option<Self> {
        Some(
            autd3_driver::datagram::FocusSTM::from_sampling_config(
                SamplingConfig::from_division_raw(msg.freq_div).ok()?,
            )
            .with_loop_behavior(autd3_driver::firmware::fpga::LoopBehavior::from_msg(
                msg.loop_behavior.as_ref()?,
            )?)
            .add_foci_from_iter(
                msg.points
                    .iter()
                    .filter_map(autd3_driver::firmware::operation::stm::ControlPoint::from_msg),
            ),
        )
    }
}

impl ToMessage for autd3_driver::datagram::ChangeFocusSTMSegment {
    type Message = ChangeFocusStmSegment;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            segment: self.segment() as _,
            transition_mode: self.transition_mode().mode() as _,
            transition_value: self.transition_mode().value(),
        }
    }
}

impl FromMessage<ChangeFocusStmSegment> for autd3_driver::datagram::ChangeFocusSTMSegment {
    fn from_msg(msg: &ChangeFocusStmSegment) -> Option<Self> {
        Some(autd3_driver::datagram::ChangeFocusSTMSegment::new(
            autd3_driver::firmware::fpga::Segment::from(Segment::try_from(msg.segment).ok()?),
            to_transition_mode(Some(msg.transition_mode), Some(msg.transition_value)).unwrap(),
        ))
    }
}
