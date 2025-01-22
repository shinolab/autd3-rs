mod clear;
mod force_fan;
mod reads_fpga_state;
mod segment;
mod silencer;
mod stm;
mod sync;

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
                datagram: Some(datagram::Datagram::GainWithSegment(GainWithSegment {
                    gain: Some(g),
                    segment: Some(self.segment as _),
                    transition_mode: self
                        .transition_mode
                        .map(|mode| mode.to_msg(geometry))
                        .transpose()?,
                })),
            },
            Some(datagram::Datagram::Modulation(m)) => Self::Message {
                datagram: Some(datagram::Datagram::ModulationWithSegment(
                    ModulationWithLoopBehavior {
                        modulation: Some(m),
                        segment: Some(self.segment as _),
                        transition_mode: self
                            .transition_mode
                            .map(|mode| mode.to_msg(geometry))
                            .transpose()?,
                        loop_behavior: None,
                    },
                )),
            },
            Some(datagram::Datagram::FociStm(stm)) => Self::Message {
                datagram: Some(datagram::Datagram::FociStmWithLoopBehavior(
                    FociStmWithLoopBehavior {
                        foci_stm: Some(stm),
                        segment: Some(self.segment as _),
                        transition_mode: self
                            .transition_mode
                            .map(|mode| mode.to_msg(geometry))
                            .transpose()?,
                        loop_behavior: None,
                    },
                )),
            },
            Some(datagram::Datagram::GainStm(stm)) => Self::Message {
                datagram: Some(datagram::Datagram::GainStmWithLoopBehavior(
                    GainStmWithLoopBehavior {
                        gain_stm: Some(stm),
                        segment: Some(self.segment as _),
                        transition_mode: self
                            .transition_mode
                            .map(|mode| mode.to_msg(geometry))
                            .transpose()?,
                        loop_behavior: None,
                    },
                )),
            },
            _ => unreachable!(),
        })
    }
}

impl<T> ToMessage for autd3_driver::datagram::WithLoopBehavior<T>
where
    T: autd3_core::datagram::DatagramL + ToMessage<Message = Datagram>,
{
    type Message = Datagram;

    fn to_msg(
        &self,
        geometry: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        let datagram = <T as ToMessage>::to_msg(self.deref(), geometry)?;
        Ok(match datagram.datagram {
            Some(datagram::Datagram::Modulation(m)) => Self::Message {
                datagram: Some(datagram::Datagram::ModulationWithSegment(
                    ModulationWithLoopBehavior {
                        modulation: Some(m),
                        segment: Some(self.segment as _),
                        transition_mode: self
                            .transition_mode
                            .map(|mode| mode.to_msg(geometry))
                            .transpose()?,
                        loop_behavior: None,
                    },
                )),
            },
            Some(datagram::Datagram::FociStm(stm)) => Self::Message {
                datagram: Some(datagram::Datagram::FociStmWithLoopBehavior(
                    FociStmWithLoopBehavior {
                        foci_stm: Some(stm),
                        segment: Some(self.segment as _),
                        transition_mode: self
                            .transition_mode
                            .map(|mode| mode.to_msg(geometry))
                            .transpose()?,
                        loop_behavior: None,
                    },
                )),
            },
            Some(datagram::Datagram::GainStm(stm)) => Self::Message {
                datagram: Some(datagram::Datagram::GainStmWithLoopBehavior(
                    GainStmWithLoopBehavior {
                        gain_stm: Some(stm),
                        segment: Some(self.segment as _),
                        transition_mode: self
                            .transition_mode
                            .map(|mode| mode.to_msg(geometry))
                            .transpose()?,
                        loop_behavior: None,
                    },
                )),
            },
            _ => unreachable!(),
        })
    }
}
