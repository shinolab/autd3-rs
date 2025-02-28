use crate::{
    AUTDProtoBufError,
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3_driver::datagram::Synchronize {
    type Message = Datagram;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(Self::Message {
            datagram: Some(datagram::Datagram::Synchronize(Synchronize {})),
        })
    }
}

impl FromMessage<Synchronize> for autd3_driver::datagram::Synchronize {
    fn from_msg(_: Synchronize) -> Result<Self, AUTDProtoBufError> {
        Ok(autd3_driver::datagram::Synchronize::new())
    }
}
