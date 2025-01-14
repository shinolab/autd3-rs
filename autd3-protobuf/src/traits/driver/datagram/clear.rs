use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

impl ToMessage for autd3_driver::datagram::Clear {
    type Message = Datagram;

    fn to_msg(&self, _: Option<&autd3_core::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram::Datagram::Clear(Clear {})),
            parallel_threshold: None,
            timeout: None,
        }
    }
}

impl FromMessage<Clear> for autd3_driver::datagram::Clear {
    fn from_msg(_: &Clear) -> Result<Self, AUTDProtoBufError> {
        Ok(autd3_driver::datagram::Clear::new())
    }
}
