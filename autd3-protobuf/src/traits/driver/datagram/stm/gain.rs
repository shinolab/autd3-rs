use autd3_driver::derive::SamplingConfig;

use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

impl<G> ToMessage for autd3_driver::datagram::GainSTM<G>
where
    G: autd3_driver::datagram::Gain + ToMessage<Message = Datagram>,
{
    type Message = GainStm;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            config: Some(self.sampling_config().to_msg(None)),
            loop_behavior: Some(self.loop_behavior().to_msg(None)),
            gains: self
                .iter()
                .filter_map(|g| match g.to_msg(None).datagram {
                    Some(datagram::Datagram::Gain(gain)) => Some(gain),
                    _ => None,
                })
                .collect(),
            mode: Some(self.mode() as _),
        }
    }
}

impl FromMessage<GainStm> for autd3_driver::datagram::GainSTM<autd3_driver::datagram::BoxedGain> {
    fn from_msg(msg: &GainStm) -> Result<Self, AUTDProtoBufError> {
        use autd3_driver::datagram::IntoBoxedGain;
        let mut stm = autd3_driver::datagram::GainSTM::new(
            SamplingConfig::from_msg(
                msg.config
                    .as_ref()
                    .ok_or(AUTDProtoBufError::DataParseError)?,
            )?,
            msg.gains
                .iter()
                .map(|gain| match &gain.gain {
                    Some(gain::Gain::Focus(msg)) => {
                        autd3::prelude::Focus::from_msg(msg).map(|g| g.into_boxed())
                    }
                    Some(gain::Gain::Bessel(msg)) => {
                        autd3::prelude::Bessel::from_msg(msg).map(|g| g.into_boxed())
                    }
                    Some(gain::Gain::Null(msg)) => {
                        autd3::prelude::Null::from_msg(msg).map(|g| g.into_boxed())
                    }
                    Some(gain::Gain::Plane(msg)) => {
                        autd3::prelude::Plane::from_msg(msg).map(|g| g.into_boxed())
                    }
                    Some(gain::Gain::Uniform(msg)) => {
                        autd3::prelude::Uniform::from_msg(msg).map(|g| g.into_boxed())
                    }
                    Some(gain::Gain::Naive(msg)) => {
                        autd3_gain_holo::Naive::from_msg(msg).map(|g| g.into_boxed())
                    }
                    Some(gain::Gain::Gs(msg)) => {
                        autd3_gain_holo::GS::from_msg(msg).map(|g| g.into_boxed())
                    }
                    Some(gain::Gain::Gspat(msg)) => {
                        autd3_gain_holo::GSPAT::from_msg(msg).map(|g| g.into_boxed())
                    }
                    Some(gain::Gain::Lm(msg)) => {
                        autd3_gain_holo::LM::from_msg(msg).map(|g| g.into_boxed())
                    }
                    Some(gain::Gain::Greedy(msg)) => {
                        autd3_gain_holo::Greedy::from_msg(msg).map(|g| g.into_boxed())
                    }
                    None => Err(AUTDProtoBufError::DataParseError),
                })
                .collect::<Result<Vec<_>, _>>()?,
        )?;
        if let Some(loop_behavior) = msg.loop_behavior.as_ref() {
            stm = stm.with_loop_behavior(autd3_driver::firmware::fpga::LoopBehavior::from_msg(
                loop_behavior,
            )?);
        }
        if let Some(mode) = msg.mode {
            stm = stm.with_mode(autd3_driver::firmware::cpu::GainSTMMode::from(
                GainStmMode::try_from(mode)?,
            ));
        }
        Ok(stm)
    }
}
