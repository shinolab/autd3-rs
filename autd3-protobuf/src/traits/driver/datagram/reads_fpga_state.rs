use autd3_driver::geometry::Device;

use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl<F: Fn(&Device) -> bool> ToMessage for autd3_driver::datagram::ReadsFPGAState<F> {
    type Message = DatagramLightweight;

    fn to_msg(&self, geometry: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram_lightweight::Datagram::ReadsFpgaState(
                ReadsFpgaState {
                    value: geometry
                        .map(|g| g.iter().map(|d| (self.f())(d)).collect::<Vec<bool>>())
                        .unwrap_or_default(),
                },
            )),
        }
    }
}

impl FromMessage<ReadsFpgaState>
    for autd3_driver::datagram::ReadsFPGAState<Box<dyn Fn(&Device) -> bool + Send + Sync + 'static>>
{
    fn from_msg(msg: &ReadsFpgaState) -> Option<Self> {
        let map = msg.value.clone();
        Some(autd3_driver::datagram::ReadsFPGAState::new(Box::new(
            move |dev| map[dev.idx()],
        )))
    }
}
