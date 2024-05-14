use autd3_driver::derive::SamplingConfig;

use crate::{pb::*, traits::*};

impl ToMessage for autd3_driver::datagram::FocusSTM {
    type Message = FocusStm;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            config: self.sampling_config().ok().map(|c| c.to_msg(None)),
            loop_behavior: Some(self.loop_behavior().to_msg(None)),
            segment: Segment::S0 as _,
            transition_mode: Some(TransitionMode::SyncIdx.into()),
            transition_value: Some(0),
            points: self.iter().map(|p| p.to_msg(None)).collect(),
        }
    }
}

impl ToMessage
    for autd3_driver::datagram::DatagramWithSegmentTransition<autd3_driver::datagram::FocusSTM>
{
    type Message = FocusStm;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            config: self.sampling_config().ok().map(|c| c.to_msg(None)),
            loop_behavior: Some(self.loop_behavior().to_msg(None)),
            segment: self.segment() as _,
            transition_mode: self.transition_mode().map(|m| m.mode() as _),
            transition_value: self.transition_mode().map(|m| m.value()),
            points: self.iter().map(|p| p.to_msg(None)).collect(),
        }
    }
}

impl FromMessage<FocusStm> for autd3_driver::datagram::FocusSTM {
    #[allow(clippy::unnecessary_cast)]
    fn from_msg(msg: &FocusStm) -> Option<Self> {
        if msg.config.is_none() {
            return None;
        }
        Some(
            autd3_driver::datagram::FocusSTM::from_sampling_config(
                SamplingConfig::from_msg(msg.config.as_ref().unwrap()).unwrap(),
            )
            .with_loop_behavior(autd3_driver::firmware::fpga::LoopBehavior::from_msg(
                msg.loop_behavior.as_ref()?,
            )?)
            .add_foci_from_iter(
                msg.points
                    .iter()
                    .filter_map(autd3_driver::datagram::ControlPoint::from_msg),
            ),
        )
    }
}
