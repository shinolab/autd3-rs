use crate::{AUTDProtoBufError, FromMessage, pb::gain::Gain};
use autd3::gain::*;
use autd3_driver::datagram::BoxedGain;
use autd3_gain_holo::*;

pub(crate) fn gain_into_boxed(msg: crate::pb::Gain) -> Result<BoxedGain, AUTDProtoBufError> {
    let gain = msg.gain.ok_or(AUTDProtoBufError::DataParseError)?;
    match gain {
        Gain::Focus(msg) => Focus::from_msg(msg).map(BoxedGain::new),
        Gain::Bessel(msg) => Bessel::from_msg(msg).map(BoxedGain::new),
        Gain::Plane(msg) => Plane::from_msg(msg).map(BoxedGain::new),
        Gain::Uniform(msg) => Uniform::from_msg(msg).map(BoxedGain::new),
        Gain::Null(msg) => Null::from_msg(msg).map(BoxedGain::new),
        Gain::Lm(msg) => LM::from_msg(msg).map(BoxedGain::new),
        Gain::Gs(msg) => GS::from_msg(msg).map(BoxedGain::new),
        Gain::Naive(msg) => Naive::from_msg(msg).map(BoxedGain::new),
        Gain::Gspat(msg) => GSPAT::from_msg(msg).map(BoxedGain::new),
        Gain::Greedy(msg) => Greedy::from_msg(msg).map(BoxedGain::new),
    }
}
