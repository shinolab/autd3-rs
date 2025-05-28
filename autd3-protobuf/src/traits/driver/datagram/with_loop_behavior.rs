use crate::{AUTDProtoBufError, pb::*, traits::DatagramLightweight};

impl<T> DatagramLightweight for autd3_driver::datagram::WithLoopBehavior<T>
where
    T: autd3_core::datagram::DatagramL + DatagramLightweight,
{
    fn into_datagram_lightweight(
        self,
        geometry: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<RawDatagram, AUTDProtoBufError> {
        let autd3_driver::datagram::WithLoopBehavior {
            inner,
            segment,
            transition_mode,
            loop_behavior,
        } = self;
        let datagram = <T as DatagramLightweight>::into_datagram_lightweight(inner, geometry)?;
        Ok(match datagram.datagram {
            Some(raw_datagram::Datagram::Modulation(m)) => RawDatagram {
                datagram: Some(raw_datagram::Datagram::WithLoopBehavior(WithLoopBehavior {
                    inner: Some(with_loop_behavior::Inner::Modulation(m)),
                    segment: segment as u8 as _,
                    transition_mode: transition_mode.map(|mode| mode.into()),
                    loop_behavior: Some(loop_behavior.into()),
                })),
            },
            Some(raw_datagram::Datagram::FociStm(stm)) => RawDatagram {
                datagram: Some(raw_datagram::Datagram::WithLoopBehavior(WithLoopBehavior {
                    inner: Some(with_loop_behavior::Inner::FociStm(stm)),
                    segment: segment as u8 as _,
                    transition_mode: transition_mode.map(|mode| mode.into()),
                    loop_behavior: Some(loop_behavior.into()),
                })),
            },
            Some(raw_datagram::Datagram::GainStm(stm)) => RawDatagram {
                datagram: Some(raw_datagram::Datagram::WithLoopBehavior(WithLoopBehavior {
                    inner: Some(with_loop_behavior::Inner::GainStm(stm)),
                    segment: segment as u8 as _,
                    transition_mode: transition_mode.map(|mode| mode.into()),
                    loop_behavior: Some(loop_behavior.into()),
                })),
            },
            _ => unreachable!(),
        })
    }
}
