use autd3_driver::{datagram::STMConfig, firmware::fpga::SamplingConfig};

use crate::{pb::*, traits::*};

seq_macro::seq!(N in 1..=8 {
    #(
        impl<C: Into<STMConfig> + Copy> ToMessage for autd3_driver::datagram::FociSTM<N, Vec<autd3_driver::datagram::ControlPoints<N>>, C> {
            type Message = FociStm~N;

            fn to_msg(&self, _: Option<&autd3_core::geometry::Geometry>) -> Result<Self::Message, AUTDProtoBufError> {
               Ok(Self::Message {
                    props: Some(FociStmProps {
                        sampling_config: Some(self.sampling_config()?.to_msg(None)?),
                    }),
                    foci: self.iter().map(|p| p.to_msg(None)).collect::<Result<Vec<_>, AUTDProtoBufError>>()?,
                })
            }
        }
    )*
});

seq_macro::seq!(N in 1..=8 {
    #(
        impl FromMessage<FociStm~N> for autd3_driver::datagram::FociSTM<N, Vec<autd3_driver::datagram::ControlPoints<N>>, SamplingConfig> {
            fn from_msg(msg: &FociStm~N) -> Result<Self, AUTDProtoBufError> {
                Ok(autd3_driver::datagram::FociSTM {
                    foci: msg.foci
                    .iter()
                    .map(|f| {
                        Ok(autd3_driver::datagram::ControlPoints::<N>{
                            points: (0..N)
                                .map(|i| autd3_driver::datagram::ControlPoint::from_msg(&f.points[i]))
                                .collect::<Result<Vec<_>, AUTDProtoBufError>>()?
                                .as_slice()
                                .try_into()
                                .unwrap(),
                                intensity: f.intensity.map(|v| autd3_driver::firmware::fpga::EmitIntensity::from_msg(&v)).transpose()?.unwrap_or(autd3_driver::firmware::fpga::EmitIntensity::MAX),
                        })
                    })
                    .collect::<Result<Vec<_>, AUTDProtoBufError>>()?,
                    config: SamplingConfig::from_msg(
                        msg.props
                            .as_ref()
                            .ok_or(AUTDProtoBufError::DataParseError)?
                            .sampling_config
                            .as_ref()
                            .ok_or(AUTDProtoBufError::DataParseError)?,
                    )?
                })
            }
        }
    )*
});
