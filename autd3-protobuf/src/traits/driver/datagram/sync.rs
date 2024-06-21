use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

impl ToMessage for autd3_driver::datagram::Synchronize {
    type Message = Datagram;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram::Datagram::Synchronize(Synchronize {})),
            parallel_threshold: None,
            timeout: None,
        }
    }
}

impl FromMessage<Synchronize> for autd3_driver::datagram::Synchronize {
    fn from_msg(_: &Synchronize) -> Result<Self, AUTDProtoBufError> {
        Ok(autd3_driver::datagram::Synchronize::new())
    }
}
