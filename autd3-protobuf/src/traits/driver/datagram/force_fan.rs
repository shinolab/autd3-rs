use autd3_core::geometry::Device;

use crate::{
    AUTDProtoBufError,
    pb::*,
    traits::{DatagramLightweight, FromMessage},
};

impl<F: Fn(&Device) -> bool> DatagramLightweight for autd3_driver::datagram::ForceFan<F> {
    fn into_datagram_lightweight(
        self,
        geometry: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<RawDatagram, AUTDProtoBufError> {
        Ok(RawDatagram {
            datagram: Some(raw_datagram::Datagram::ForceFan(ForceFan {
                value: geometry
                    .unwrap()
                    .iter()
                    .map(|d| (self.f)(d))
                    .collect::<Vec<bool>>(),
            })),
        })
    }
}

impl FromMessage<ForceFan>
    for autd3_driver::datagram::ForceFan<Box<dyn Fn(&Device) -> bool + Send + Sync + 'static>>
{
    fn from_msg(msg: ForceFan) -> Result<Self, AUTDProtoBufError> {
        let map = msg.value.clone();
        Ok(autd3_driver::datagram::ForceFan::new(Box::new(
            move |dev| map[dev.idx()],
        )))
    }
}
