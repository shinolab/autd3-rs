mod clear;
mod force_fan;
mod reads_fpga_state;
mod segment;
mod silencer;
mod stm;
mod sync;

use std::ops::Deref;

use crate::{pb::*, traits::ToMessage};

impl<T> ToMessage for autd3_driver::datagram::DatagramWithSegment<T>
where
    T: autd3_driver::datagram::DatagramS + ToMessage<Message = Datagram>,
{
    type Message = Datagram;

    fn to_msg(&self, geometry: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        let datagram = <T as ToMessage>::to_msg(self.deref(), geometry);

        match datagram.datagram {
            Some(datagram::Datagram::Gain(g)) => Self::Message {
                datagram: Some(datagram::Datagram::GainWithSegment(GainWithSegment {
                    gain: Some(g),
                    segment: self.segment() as _,
                    transition_mode: self.transition_mode().map(|mode| mode.to_msg(geometry)),
                })),
                timeout: None,
                parallel_threshold: None,
            },
            Some(datagram::Datagram::Modulation(m)) => Self::Message {
                datagram: Some(datagram::Datagram::ModulationWithSegment(
                    ModulationWithSegment {
                        modulation: Some(m),
                        segment: self.segment() as _,
                        transition_mode: self.transition_mode().map(|mode| mode.to_msg(geometry)),
                    },
                )),
                timeout: None,
                parallel_threshold: None,
            },
            Some(datagram::Datagram::FociStm(stm)) => Self::Message {
                datagram: Some(datagram::Datagram::FociStmWithSegment(FociStmWithSegment {
                    foci_stm: Some(stm),
                    segment: self.segment() as _,
                    transition_mode: self.transition_mode().map(|mode| mode.to_msg(geometry)),
                })),
                timeout: None,
                parallel_threshold: None,
            },
            Some(datagram::Datagram::GainStm(stm)) => Self::Message {
                datagram: Some(datagram::Datagram::GainStmWithSegment(GainStmWithSegment {
                    gain_stm: Some(stm),
                    segment: self.segment() as _,
                    transition_mode: self.transition_mode().map(|mode| mode.to_msg(geometry)),
                })),
                timeout: None,
                parallel_threshold: None,
            },
            _ => unreachable!(),
        }
    }
}
