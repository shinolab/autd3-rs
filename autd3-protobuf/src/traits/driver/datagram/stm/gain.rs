use autd3_driver::derive::SamplingConfig;

use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl<G> ToMessage for autd3_driver::datagram::GainSTM<G>
where
    G: autd3_driver::datagram::Gain + ToMessage<Message = DatagramLightweight>,
{
    type Message = GainStm;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            config: Some(self.sampling_config().to_msg(None)),
            loop_behavior: Some(self.loop_behavior().to_msg(None)),
            segment: Segment::S0 as _,
            transition_mode: Some(TransitionMode::SyncIdx.into()),
            transition_value: Some(0),
            gains: self
                .iter()
                .filter_map(|g| match g.to_msg(None).datagram {
                    Some(datagram_lightweight::Datagram::Gain(gain)) => Some(gain),
                    _ => None,
                })
                .collect(),
        }
    }
}

impl<G> ToMessage
    for autd3_driver::datagram::DatagramWithSegmentTransition<autd3_driver::datagram::GainSTM<G>>
where
    G: autd3_driver::datagram::Gain + ToMessage<Message = DatagramLightweight>,
{
    type Message = GainStm;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            config: Some(self.sampling_config().to_msg(None)),
            loop_behavior: Some(self.loop_behavior().to_msg(None)),
            segment: self.segment() as _,
            transition_mode: self.transition_mode().map(|m| m.mode() as _),
            transition_value: self.transition_mode().map(|m| m.value()),
            gains: self
                .iter()
                .filter_map(|g| match g.to_msg(None).datagram {
                    Some(datagram_lightweight::Datagram::Gain(gain)) => Some(gain),
                    _ => None,
                })
                .collect(),
        }
    }
}

impl FromMessage<GainStm>
    for autd3_driver::datagram::GainSTM<Box<dyn autd3_driver::datagram::Gain + Send + Sync>>
{
    #[allow(clippy::unnecessary_cast)]
    fn from_msg(msg: &GainStm) -> Option<Self> {
        Some(
            autd3_driver::datagram::GainSTM::from_sampling_config(
                SamplingConfig::from_msg(msg.config.as_ref().unwrap()).unwrap(),
                msg.gains.iter().filter_map(|gain| match &gain.gain {
                    Some(gain::Gain::Focus(msg)) => {
                        autd3::prelude::Focus::from_msg(msg).map(|g| Box::new(g) as Box<_>)
                    }
                    Some(gain::Gain::Bessel(msg)) => {
                        autd3::prelude::Bessel::from_msg(msg).map(|g| Box::new(g) as Box<_>)
                    }
                    Some(gain::Gain::Null(msg)) => {
                        autd3::prelude::Null::from_msg(msg).map(|g| Box::new(g) as Box<_>)
                    }
                    Some(gain::Gain::Plane(msg)) => {
                        autd3::prelude::Plane::from_msg(msg).map(|g| Box::new(g) as Box<_>)
                    }
                    Some(gain::Gain::Uniform(msg)) => {
                        autd3::prelude::Uniform::from_msg(msg).map(|g| Box::new(g) as Box<_>)
                    }
                    Some(gain::Gain::Sdp(msg)) => {
                        autd3_gain_holo::SDP::from_msg(msg).map(|g| Box::new(g) as Box<_>)
                    }
                    Some(gain::Gain::Naive(msg)) => {
                        autd3_gain_holo::Naive::from_msg(msg).map(|g| Box::new(g) as Box<_>)
                    }
                    Some(gain::Gain::Gs(msg)) => {
                        autd3_gain_holo::GS::from_msg(msg).map(|g| Box::new(g) as Box<_>)
                    }
                    Some(gain::Gain::Gspat(msg)) => {
                        autd3_gain_holo::GSPAT::from_msg(msg).map(|g| Box::new(g) as Box<_>)
                    }
                    Some(gain::Gain::Lm(msg)) => {
                        autd3_gain_holo::LM::from_msg(msg).map(|g| Box::new(g) as Box<_>)
                    }
                    Some(gain::Gain::Greedy(msg)) => {
                        autd3_gain_holo::Greedy::from_msg(msg).map(|g| Box::new(g) as Box<_>)
                    }
                    None => None,
                }),
            )
            .with_loop_behavior(autd3_driver::firmware::fpga::LoopBehavior::from_msg(
                msg.loop_behavior.as_ref()?,
            )?),
        )
    }
}
