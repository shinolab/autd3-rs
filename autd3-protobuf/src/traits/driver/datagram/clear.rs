use crate::{
    AUTDProtoBufError,
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3_driver::datagram::Clear {
    type Message = Datagram;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(Self::Message {
            datagram: Some(datagram::Datagram::Clear(Clear {})),
        })
    }
}

impl FromMessage<Clear> for autd3_driver::datagram::Clear {
    fn from_msg(_: Clear) -> Result<Self, AUTDProtoBufError> {
        Ok(autd3_driver::datagram::Clear::new())
    }
}
