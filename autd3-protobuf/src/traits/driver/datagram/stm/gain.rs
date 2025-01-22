use autd3_driver::{datagram::STMConfig, firmware::fpga::SamplingConfig};

use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

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
            props: Some(GainStmProps {
                mode: Some(self.option.mode as _),
                sampling_config: Some(self.sampling_config()?.to_msg(None)?),
            }),
            gains: self
                .iter()
                .map(|g| match g.to_msg(None)?.datagram {
                    Some(datagram::Datagram::Gain(gain)) => Ok(gain),
                    _ => unreachable!(),
                })
                .collect::<Result<_, AUTDProtoBufError>>()?,
        })
    }
}

impl FromMessage<GainStm>
    for autd3_driver::datagram::GainSTM<Vec<autd3_driver::datagram::BoxedGain>, SamplingConfig>
{
    fn from_msg(msg: &GainStm) -> Result<Self, AUTDProtoBufError> {
        use autd3_driver::datagram::IntoBoxedGain;
        Ok(autd3_driver::datagram::GainSTM {
            gains: msg
                .gains
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
            config: SamplingConfig::from_msg(
                msg.props
                    .as_ref()
                    .ok_or(AUTDProtoBufError::DataParseError)?
                    .sampling_config
                    .as_ref()
                    .ok_or(AUTDProtoBufError::DataParseError)?,
            )?,
            option: autd3_driver::datagram::GainSTMOption {
                mode: autd3_driver::firmware::cpu::GainSTMMode::from(
                    msg.props
                        .as_ref()
                        .ok_or(AUTDProtoBufError::DataParseError)?
                        .mode
                        .map(GainStmMode::try_from)
                        .transpose()?
                        .unwrap_or(GainStmMode::PhaseIntensityFull),
                ),
            },
        })
    }
}
