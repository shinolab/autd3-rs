use autd3_driver::{datagram::STMConfig, firmware::fpga::SamplingConfig};

use crate::{
    AUTDProtoBufError,
    pb::*,
    traits::{DatagramLightweight, FromMessage},
};

impl FromMessage<i32> for autd3_driver::firmware::cpu::GainSTMMode {
    fn from_msg(msg: i32) -> Result<Self, AUTDProtoBufError> {
        Ok(GainStmMode::try_from(msg)?.into())
    }
}

impl From<autd3_driver::datagram::GainSTMOption> for GainStmOption {
    fn from(value: autd3_driver::datagram::GainSTMOption) -> Self {
        Self {
            mode: Some(GainStmMode::from(value.mode) as _),
        }
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

impl<C: Into<STMConfig> + Copy> DatagramLightweight
    for autd3_driver::datagram::GainSTM<Vec<Gain>, C>
{
    fn into_datagram_lightweight(
        self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<RawDatagram, AUTDProtoBufError> {
        let sampling_config = self.sampling_config()?.into();
        let autd3_driver::datagram::GainSTM { gains, option, .. } = self;
        Ok(RawDatagram {
            datagram: Some(raw_datagram::Datagram::GainStm(GainStm {
                gains: gains.into_iter().collect(),
                option: Some(GainStmOption::from(option) as _),
                sampling_config: Some(sampling_config),
            })),
        })
    }
}

impl FromMessage<GainStm>
    for autd3_driver::datagram::GainSTM<Vec<autd3_driver::datagram::BoxedGain>, SamplingConfig>
{
    fn from_msg(msg: GainStm) -> Result<Self, AUTDProtoBufError> {
        use autd3_driver::datagram::BoxedGain;
        Ok(autd3_driver::datagram::GainSTM {
            gains: msg
                .gains
                .into_iter()
                .map(|gain| match gain.gain {
                    Some(gain::Gain::Focus(msg)) => {
                        autd3::prelude::Focus::from_msg(msg).map(BoxedGain::new)
                    }
                    Some(gain::Gain::Bessel(msg)) => {
                        autd3::prelude::Bessel::from_msg(msg).map(BoxedGain::new)
                    }
                    Some(gain::Gain::Null(msg)) => {
                        autd3::prelude::Null::from_msg(msg).map(BoxedGain::new)
                    }
                    Some(gain::Gain::Plane(msg)) => {
                        autd3::prelude::Plane::from_msg(msg).map(BoxedGain::new)
                    }
                    Some(gain::Gain::Uniform(msg)) => {
                        autd3::prelude::Uniform::from_msg(msg).map(BoxedGain::new)
                    }
                    Some(gain::Gain::Naive(msg)) => {
                        autd3_gain_holo::Naive::from_msg(msg).map(BoxedGain::new)
                    }
                    Some(gain::Gain::Gs(msg)) => {
                        autd3_gain_holo::GS::from_msg(msg).map(BoxedGain::new)
                    }
                    Some(gain::Gain::Gspat(msg)) => {
                        autd3_gain_holo::GSPAT::from_msg(msg).map(BoxedGain::new)
                    }
                    Some(gain::Gain::Lm(msg)) => {
                        autd3_gain_holo::LM::from_msg(msg).map(BoxedGain::new)
                    }
                    Some(gain::Gain::Greedy(msg)) => {
                        autd3_gain_holo::Greedy::from_msg(msg).map(BoxedGain::new)
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
