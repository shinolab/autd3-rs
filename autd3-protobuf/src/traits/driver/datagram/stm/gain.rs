use autd3_driver::derive::SamplingConfiguration;

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
            freq_div: self.sampling_config().unwrap().frequency_division(),
            loop_behavior: Some(self.loop_behavior().to_msg(None)),
            segment: Segment::S0 as _,
            update_segment: true,
            gains: self
                .gains()
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
    for autd3_driver::datagram::DatagramWithSegment<autd3_driver::datagram::GainSTM<G>>
where
    G: autd3_driver::datagram::Gain + ToMessage<Message = DatagramLightweight>,
{
    type Message = GainStm;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            freq_div: self.sampling_config().unwrap().frequency_division(),
            loop_behavior: Some(self.loop_behavior().to_msg(None)),
            segment: self.segment() as _,
            update_segment: self.update_segment(),
            gains: self
                .gains()
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
    for autd3_driver::datagram::GainSTM<Box<dyn autd3_driver::datagram::Gain + Send + 'static>>
{
    #[allow(clippy::unnecessary_cast)]
    fn from_msg(msg: &GainStm) -> Option<Self> {
        autd3_driver::datagram::GainSTM::from_sampling_config(
            SamplingConfiguration::from_frequency_division(msg.freq_div).ok()?,
        )
        .with_loop_behavior(autd3_driver::fpga::LoopBehavior::from_msg(
            msg.loop_behavior.as_ref()?,
        )?)
        .add_gains_from_iter(msg.gains.iter().filter_map(|gain| match &gain.gain {
            Some(gain::Gain::Focus(msg)) => autd3::prelude::Focus::from_msg(msg).map(|g| {
                let g: Box<dyn autd3_driver::datagram::Gain + Send + 'static> = Box::new(g);
                g
            }),
            Some(gain::Gain::Bessel(msg)) => autd3::prelude::Bessel::from_msg(msg).map(|g| {
                let g: Box<dyn autd3_driver::datagram::Gain + Send + 'static> = Box::new(g);
                g
            }),
            Some(gain::Gain::Null(msg)) => autd3::prelude::Null::from_msg(msg).map(|g| {
                let g: Box<dyn autd3_driver::datagram::Gain + Send + 'static> = Box::new(g);
                g
            }),
            Some(gain::Gain::Plane(msg)) => autd3::prelude::Plane::from_msg(msg).map(|g| {
                let g: Box<dyn autd3_driver::datagram::Gain + Send + 'static> = Box::new(g);
                g
            }),
            Some(gain::Gain::Uniform(msg)) => autd3::prelude::Uniform::from_msg(msg).map(|g| {
                let g: Box<dyn autd3_driver::datagram::Gain + Send + 'static> = Box::new(g);
                g
            }),
            Some(gain::Gain::Sdp(msg)) => autd3_gain_holo::SDP::from_msg(msg).map(|g| {
                let g: Box<dyn autd3_driver::datagram::Gain + Send + 'static> = Box::new(g);
                g
            }),
            Some(gain::Gain::Naive(msg)) => autd3_gain_holo::Naive::from_msg(msg).map(|g| {
                let g: Box<dyn autd3_driver::datagram::Gain + Send + 'static> = Box::new(g);
                g
            }),
            Some(gain::Gain::Gs(msg)) => autd3_gain_holo::GS::from_msg(msg).map(|g| {
                let g: Box<dyn autd3_driver::datagram::Gain + Send + 'static> = Box::new(g);
                g
            }),
            Some(gain::Gain::Gspat(msg)) => autd3_gain_holo::GSPAT::from_msg(msg).map(|g| {
                let g: Box<dyn autd3_driver::datagram::Gain + Send + 'static> = Box::new(g);
                g
            }),
            Some(gain::Gain::Lm(msg)) => autd3_gain_holo::LM::from_msg(msg).map(|g| {
                let g: Box<dyn autd3_driver::datagram::Gain + Send + 'static> = Box::new(g);
                g
            }),
            Some(gain::Gain::Greedy(msg)) => autd3_gain_holo::Greedy::from_msg(msg).map(|g| {
                let g: Box<dyn autd3_driver::datagram::Gain + Send + 'static> = Box::new(g);
                g
            }),
            None => None,
        }))
        .ok()
    }
}

impl ToMessage for autd3_driver::datagram::ChangeGainSTMSegment {
    type Message = ChangeGainStmSegment;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            segment: self.segment() as _,
        }
    }
}

impl FromMessage<ChangeGainStmSegment> for autd3_driver::datagram::ChangeGainSTMSegment {
    fn from_msg(msg: &ChangeGainStmSegment) -> Option<Self> {
        Some(autd3_driver::datagram::ChangeGainSTMSegment::new(
            autd3_driver::fpga::Segment::from(Segment::try_from(msg.segment).ok()?),
        ))
    }
}
