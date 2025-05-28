use autd3_driver::{datagram::STMConfig, firmware::fpga::SamplingConfig};

use crate::{pb::*, traits::*};

impl<const N: usize, C: Into<STMConfig> + Copy> DatagramLightweight
    for autd3_driver::datagram::FociSTM<N, Vec<autd3_driver::datagram::ControlPoints<N>>, C>
{
    fn into_datagram_lightweight(
        self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<RawDatagram, AUTDProtoBufError> {
        let sampling_config = self.sampling_config()?.into();
        let autd3_driver::datagram::FociSTM { foci, .. } = self;
        Ok(RawDatagram {
            datagram: Some(raw_datagram::Datagram::FociStm(FociStm {
                foci: foci.into_iter().map(|p| p.into()).collect(),
                sampling_config: Some(sampling_config),
            })),
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
