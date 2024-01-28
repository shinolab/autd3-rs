use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3_driver::datagram::Clear {
    type Message = DatagramLightweight;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram_lightweight::Datagram::Clear(Clear {})),
        }
    }
}

impl FromMessage<Clear> for autd3_driver::datagram::Clear {
    fn from_msg(_: &Clear) -> Option<Self> {
        Some(autd3_driver::datagram::Clear::new())
    }
}
