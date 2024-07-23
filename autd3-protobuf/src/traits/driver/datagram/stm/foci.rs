use autd3_driver::derive::SamplingConfig;

use crate::{pb::*, traits::*};

seq_macro::seq!(N in 1..=8 {
    #(
        impl ToMessage for autd3_driver::datagram::FociSTM<N> {
            type Message = FociStm~N;

            fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
                Self::Message {
                    props: Some(FociStmProps {
                        config: Some(self.sampling_config().to_msg(None)),
                        loop_behavior: Some(self.loop_behavior().to_msg(None)),
                    }),
                    foci: self.iter().map(|p| p.to_msg(None)).collect(),
                }
            }
        }
    )*
});

seq_macro::seq!(N in 1..=8 {
    #(
        impl FromMessage<FociStm~N> for autd3_driver::datagram::FociSTM<N> {
            fn from_msg(msg: &FociStm~N) -> Result<Self, AUTDProtoBufError> {
                let props = msg
                    .props
                    .as_ref()
                    .ok_or(AUTDProtoBufError::DataParseError)?;

                let mut stm = autd3_driver::datagram::FociSTM::new(
                    SamplingConfig::from_msg(
                        props
                            .config
                            .as_ref()
                            .ok_or(AUTDProtoBufError::DataParseError)?,
                    )?,
                    msg.foci
                        .iter()
                        .map(|f| {
                            let mut c = autd3_driver::defined::ControlPoints::<N>::new(
                                (0..N)
                                    .map(|i| autd3_driver::defined::ControlPoint::from_msg(&f.points[i]))
                                    .collect::<Result<Vec<_>, AUTDProtoBufError>>()?
                                    .as_slice()
                                    .try_into()
                                    .unwrap(),
                            );
                            if let Some(intensity) = f.intensity.as_ref() {
                                c = c.with_intensity(
                                    autd3_driver::firmware::fpga::EmitIntensity::from_msg(intensity)?,
                                );
                            }
                            Ok(c)
                        })
                        .collect::<Result<Vec<_>, AUTDProtoBufError>>()?,
                );
                if let Some(loop_behavior) = props.loop_behavior.as_ref() {
                    stm = stm.with_loop_behavior(autd3_driver::firmware::fpga::LoopBehavior::from_msg(
                        loop_behavior,
                    )?);
                }
                Ok(stm)
            }
        }
    )*
});
