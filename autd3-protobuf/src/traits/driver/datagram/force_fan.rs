use autd3_core::geometry::Device;

use crate::{
    AUTDProtoBufError,
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl<F: Fn(&Device) -> bool> ToMessage for autd3_driver::datagram::ForceFan<F> {
    type Message = Datagram;

    fn to_msg(
        &self,
        geometry: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(Self::Message {
            datagram: Some(datagram::Datagram::ForceFan(ForceFan {
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
