use autd3_driver::geometry::Device;

use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

impl<F: Fn(&Device) -> bool> ToMessage for autd3_driver::datagram::ForceFan<F> {
    type Message = Datagram;

    fn to_msg(&self, geometry: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram::Datagram::ForceFan(ForceFan {
                value: geometry
                    .map(|g| g.iter().map(|d| (self.f())(d)).collect::<Vec<bool>>())
                    .unwrap_or_default(),
            })),
            timeout: None,
            parallel_threshold: None,
        }
    }
}

impl FromMessage<ForceFan>
    for autd3_driver::datagram::ForceFan<Box<dyn Fn(&Device) -> bool + Send + Sync + 'static>>
{
    fn from_msg(msg: &ForceFan) -> Result<Self, AUTDProtoBufError> {
        let map = msg.value.clone();
        Ok(autd3_driver::datagram::ForceFan::new(Box::new(
            move |dev| map[dev.idx()],
        )))
    }
}
