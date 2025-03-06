use crate::{
    AUTDProtoBufError,
    pb::*,
    traits::{DatagramLightweight, FromMessage},
};

impl DatagramLightweight for autd3_driver::datagram::Clear {
    fn into_datagram_lightweight(
        self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Datagram, AUTDProtoBufError> {
        Ok(Datagram {
            datagram: Some(datagram::Datagram::Clear(Clear {})),
        })
    }
}

impl FromMessage<Clear> for autd3_driver::datagram::Clear {
    fn from_msg(_: Clear) -> Result<Self, AUTDProtoBufError> {
        Ok(autd3_driver::datagram::Clear::new())
    }
}
