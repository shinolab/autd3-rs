use crate::{AUTDProtoBufError, pb::*, traits::DatagramLightweight};

impl<T> DatagramLightweight for autd3_driver::datagram::WithSegment<T>
where
    T: autd3_core::datagram::DatagramS + DatagramLightweight,
{
    fn into_datagram_lightweight(
        self,
        geometry: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Datagram, AUTDProtoBufError> {
        let autd3_driver::datagram::WithSegment {
            inner,
            segment,
            transition_mode,
        } = self;
        let datagram = <T as DatagramLightweight>::into_datagram_lightweight(inner, geometry)?;
        Ok(match datagram.datagram {
            Some(datagram::Datagram::Gain(g)) => Datagram {
                datagram: Some(datagram::Datagram::WithSegment(WithSegment {
                    inner: Some(with_segment::Inner::Gain(g)),
                    segment: segment as u8 as _,
                    transition_mode: transition_mode.map(|mode| mode.into()),
                })),
            },
            Some(datagram::Datagram::Modulation(m)) => Datagram {
                datagram: Some(datagram::Datagram::WithSegment(WithSegment {
                    inner: Some(with_segment::Inner::Modulation(m)),
                    segment: segment as u8 as _,
                    transition_mode: transition_mode.map(|mode| mode.into()),
                })),
            },
            Some(datagram::Datagram::FociStm(stm)) => Datagram {
                datagram: Some(datagram::Datagram::WithSegment(WithSegment {
                    inner: Some(with_segment::Inner::FociStm(stm)),
                    segment: segment as u8 as _,
                    transition_mode: transition_mode.map(|mode| mode.into()),
                })),
            },
            Some(datagram::Datagram::GainStm(stm)) => Datagram {
                datagram: Some(datagram::Datagram::WithSegment(WithSegment {
                    inner: Some(with_segment::Inner::GainStm(stm)),
                    segment: segment as u8 as _,
                    transition_mode: transition_mode.map(|mode| mode.into()),
                })),
            },
            _ => unreachable!(),
        })
    }
}
