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
            start_idx: self.start_idx().map(|i| i as i32).unwrap_or(-1),
            finish_idx: self.finish_idx().map(|i| i as i32).unwrap_or(-1),
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
        .with_start_idx(match msg.start_idx {
            -1 => None,
            idx => Some(idx as u16),
        })
        .with_finish_idx(match msg.finish_idx {
            -1 => None,
            idx => Some(idx as u16),
        })
        .add_foci_from_iter(
            msg.points
                .iter()
                .filter_map(|p| autd3_driver::operation::stm::ControlPoint::from_msg(p)),
        )
        .ok()
    }
}
