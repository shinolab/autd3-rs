use autd3_driver::geometry::Device;

use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl<F: Fn(&Device) -> bool> ToMessage for autd3_driver::datagram::ConfigureReadsFPGAState<F> {
    type Message = DatagramLightweight;

    fn to_msg(&self, geometry: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram_lightweight::Datagram::ReadsFpgaState(
                ConfigureReadsFpgaState {
                    value: geometry
                        .map(|g| g.iter().map(|d| (self.f())(d)).collect::<Vec<bool>>())
                        .unwrap_or_default(),
                },
            )),
        }
    }
}

impl FromMessage<ConfigureReadsFpgaState>
    for autd3_driver::datagram::ConfigureReadsFPGAState<
        Box<dyn Fn(&Device) -> bool + Send + 'static>,
    >
{
    fn from_msg(msg: &ConfigureReadsFpgaState) -> Option<Self> {
        let map = msg.value.clone();
        Some(autd3_driver::datagram::ConfigureReadsFPGAState::new(
            Box::new(move |dev| map[dev.idx()]),
        ))
    }
}
