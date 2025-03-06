use autd3_core::geometry::Device;

use crate::{
    AUTDProtoBufError,
    pb::*,
    traits::{DatagramLightweight, FromMessage},
};

impl<F: Fn(&Device) -> bool> DatagramLightweight for autd3_driver::datagram::ReadsFPGAState<F> {
    fn into_datagram_lightweight(
        self,
        geometry: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Datagram, AUTDProtoBufError> {
        Ok(Datagram {
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
