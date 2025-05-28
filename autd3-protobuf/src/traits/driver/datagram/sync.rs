use crate::{
    AUTDProtoBufError,
    pb::*,
    traits::{DatagramLightweight, FromMessage},
};

impl DatagramLightweight for autd3_driver::datagram::Synchronize {
    fn into_datagram_lightweight(
        self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<RawDatagram, AUTDProtoBufError> {
        Ok(RawDatagram {
            datagram: Some(raw_datagram::Datagram::Synchronize(Synchronize {})),
        })
    }
}

impl FromMessage<Synchronize> for autd3_driver::datagram::Synchronize {
    fn from_msg(_: Synchronize) -> Result<Self, AUTDProtoBufError> {
        Ok(autd3_driver::datagram::Synchronize::new())
    }
}
