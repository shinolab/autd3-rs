use std::ops::Deref;

use crate::{pb::*, traits::ToMessage, AUTDProtoBufError};

impl<T> ToMessage for autd3_driver::datagram::WithSegment<T>
where
    T: autd3_core::datagram::DatagramS + ToMessage<Message = Datagram>,
{
    type Message = Datagram;

    fn to_msg(
        &self,
        geometry: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        let datagram = <T as ToMessage>::to_msg(self.deref(), geometry)?;
        Ok(match datagram.datagram {
            Some(datagram::Datagram::Gain(g)) => Self::Message {
                datagram: Some(datagram::Datagram::WithSegment(WithSegment {
                    inner: Some(with_segment::Inner::Gain(g)),
                    segment: self.segment as u8 as _,
                    transition_mode: self
                        .transition_mode
                        .map(|mode| mode.to_msg(geometry))
                        .transpose()?,
                })),
            },
            Some(datagram::Datagram::Modulation(m)) => Self::Message {
                datagram: Some(datagram::Datagram::WithSegment(WithSegment {
                    inner: Some(with_segment::Inner::Modulation(m)),
                    segment: self.segment as u8 as _,
                    transition_mode: self
                        .transition_mode
                        .map(|mode| mode.to_msg(geometry))
                        .transpose()?,
                })),
            },
            Some(datagram::Datagram::FociStm(stm)) => Self::Message {
                datagram: Some(datagram::Datagram::WithSegment(WithSegment {
                    inner: Some(with_segment::Inner::FociStm(stm)),
                    segment: self.segment as u8 as _,
                    transition_mode: self
                        .transition_mode
                        .map(|mode| mode.to_msg(geometry))
                        .transpose()?,
                })),
            },
            Some(datagram::Datagram::GainStm(stm)) => Self::Message {
                datagram: Some(datagram::Datagram::WithSegment(WithSegment {
                    inner: Some(with_segment::Inner::GainStm(stm)),
                    segment: self.segment as u8 as _,
                    transition_mode: self
                        .transition_mode
                        .map(|mode| mode.to_msg(geometry))
                        .transpose()?,
                })),
            },
            _ => unreachable!(),
        })
    }
}
