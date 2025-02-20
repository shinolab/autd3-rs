use autd3_driver::{datagram::STMConfig, firmware::fpga::SamplingConfig};

use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

impl ToMessage for autd3_driver::datagram::GainSTMOption {
    type Message = GainStmOption;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(Self::Message {
            mode: Some(self.mode as u8 as _),
        })
    }
}

impl ToMessage for autd3_driver::firmware::cpu::GainSTMMode {
    type Message = i32;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(GainStmMode::from(*self) as _)
    }
}

impl FromMessage<i32> for autd3_driver::firmware::cpu::GainSTMMode {
    fn from_msg(msg: i32) -> Result<Self, AUTDProtoBufError> {
        Ok(GainStmMode::try_from(msg)?.into())
    }
}

impl FromMessage<GainStmOption> for autd3_driver::datagram::GainSTMOption {
    fn from_msg(msg: GainStmOption) -> Result<Self, AUTDProtoBufError> {
        Ok(autd3_driver::datagram::GainSTMOption {
            mode: msg
                .mode
                .map(autd3_driver::firmware::cpu::GainSTMMode::from_msg)
                .transpose()?
                .unwrap_or(autd3_driver::datagram::GainSTMOption::default().mode),
        })
    }
}

impl<G, C: Into<STMConfig> + Copy> ToMessage for autd3_driver::datagram::GainSTM<Vec<G>, C>
where
    G: autd3_core::gain::Gain + ToMessage<Message = Datagram>,
{
    type Message = GainStm;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(Self::Message {
            gains: self
                .iter()
                .map(|g| match g.to_msg(None)?.datagram {
                    Some(datagram::Datagram::Gain(gain)) => Ok(gain),
                    _ => unreachable!(),
                })
                .collect::<Result<_, AUTDProtoBufError>>()?,
            sampling_config: Some(self.sampling_config()?.to_msg(None)?),
            option: Some(self.option.to_msg(None)?),
        })
    }
}

impl FromMessage<GainStm>
    for autd3_driver::datagram::GainSTM<Vec<autd3_driver::datagram::BoxedGain>, SamplingConfig>
{
    fn from_msg(msg: GainStm) -> Result<Self, AUTDProtoBufError> {
        use autd3_driver::datagram::IntoBoxedGain;
        Ok(autd3_driver::datagram::GainSTM {
            gains: msg
                .gains
                .into_iter()
                .map(|gain| match gain.gain {
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
            config: SamplingConfig::from_msg(
                msg.sampling_config
                    .ok_or(AUTDProtoBufError::DataParseError)?,
            )?,
            option: autd3_driver::datagram::GainSTMOption::from_msg(
                msg.option.ok_or(AUTDProtoBufError::DataParseError)?,
            )?,
        })
    }
}
