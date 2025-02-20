use autd3_driver::{datagram::STMConfig, firmware::fpga::SamplingConfig};

use crate::{pb::*, traits::*};

impl<const N: usize, C: Into<STMConfig> + Copy> ToMessage
    for autd3_driver::datagram::FociSTM<N, Vec<autd3_driver::datagram::ControlPoints<N>>, C>
{
    type Message = FociStm;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(Self::Message {
            foci: self
                .iter()
                .map(|p| p.to_msg(None))
                .collect::<Result<Vec<_>, AUTDProtoBufError>>()?,
            sampling_config: Some(self.sampling_config()?.to_msg(None)?),
        })
    }
}

impl<const N: usize> FromMessage<FociStm>
    for autd3_driver::datagram::FociSTM<
        N,
        Vec<autd3_driver::datagram::ControlPoints<N>>,
        SamplingConfig,
    >
{
    fn from_msg(msg: FociStm) -> Result<Self, AUTDProtoBufError> {
        Ok(autd3_driver::datagram::FociSTM {
            foci: msg
                .foci
                .into_iter()
                .map(autd3_driver::datagram::ControlPoints::<N>::from_msg)
                .collect::<Result<Vec<_>, AUTDProtoBufError>>()?,
            config: SamplingConfig::from_msg(
                msg.sampling_config
                    .ok_or(AUTDProtoBufError::DataParseError)?,
            )?,
        })
    }
}
