use autd3_core::geometry::Device;

use crate::{
    AUTDProtoBufError,
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl<F: Fn(&Device) -> bool> ToMessage for autd3_driver::datagram::ReadsFPGAState<F> {
    type Message = Datagram;

    fn to_msg(
        &self,
        geometry: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(Self::Message {
            datagram: Some(datagram::Datagram::ReadsFpgaState(ReadsFpgaState {
                value: geometry
                    .unwrap()
                    .iter()
                    .map(|d| (self.f)(d))
                    .collect::<Vec<bool>>(),
            })),
        })
    }
}

impl FromMessage<ReadsFpgaState>
    for autd3_driver::datagram::ReadsFPGAState<Box<dyn Fn(&Device) -> bool + Send + Sync + 'static>>
{
    fn from_msg(msg: ReadsFpgaState) -> Result<Self, AUTDProtoBufError> {
        let map = msg.value.clone();
        Ok(autd3_driver::datagram::ReadsFPGAState::new(Box::new(
            move |dev| map[dev.idx()],
        )))
    }
}
