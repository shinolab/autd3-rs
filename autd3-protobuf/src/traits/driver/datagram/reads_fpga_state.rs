use autd3_driver::geometry::Device;

use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

impl<F: Fn(&Device) -> bool> ToMessage for autd3_driver::datagram::ReadsFPGAState<F> {
    type Message = Datagram;

    fn to_msg(&self, geometry: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram::Datagram::ReadsFpgaState(ReadsFpgaState {
                value: geometry
                    .map(|g| g.iter().map(|d| (self.f())(d)).collect::<Vec<bool>>())
                    .unwrap_or_default(),
            })),
            timeout: None,
            parallel_threshold: None,
        }
    }
}

impl FromMessage<ReadsFpgaState>
    for autd3_driver::datagram::ReadsFPGAState<Box<dyn Fn(&Device) -> bool + Send + Sync + 'static>>
{
    fn from_msg(msg: &ReadsFpgaState) -> Result<Self, AUTDProtoBufError> {
        let map = msg.value.clone();
        Ok(autd3_driver::datagram::ReadsFPGAState::new(Box::new(
            move |dev| map[dev.idx()],
        )))
    }
}
