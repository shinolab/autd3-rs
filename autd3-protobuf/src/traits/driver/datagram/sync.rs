use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3_driver::datagram::Synchronize {
    type Message = DatagramLightweight;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram_lightweight::Datagram::Synchronize(Synchronize {})),
        }
    }
}

impl FromMessage<Synchronize> for autd3_driver::datagram::Synchronize {
    fn from_msg(_: &Synchronize) -> Option<Self> {
        Some(autd3_driver::datagram::Synchronize::new())
    }
}
