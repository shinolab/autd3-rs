use autd3_driver::geometry::Device;

use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl<F: Fn(&Device) -> Option<&autd3_driver::geometry::Transducer>> ToMessage
    for autd3_driver::datagram::ConfigureDebugOutputIdx<F>
{
    type Message = DatagramLightweight;

    fn to_msg(&self, geometry: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram_lightweight::Datagram::Debug(
                ConfigureDebugOutputIdx {
                    value: geometry
                        .map(|g| {
                            g.iter()
                                .map(|d| (self.f())(d).map_or_else(|| -1, |tr| tr.idx() as i32))
                                .collect::<Vec<i32>>()
                        })
                        .unwrap_or_default(),
                },
            )),
        }
    }
}

impl FromMessage<ConfigureDebugOutputIdx>
    for autd3_driver::datagram::ConfigureDebugOutputIdx<
        Box<dyn Fn(&Device) -> Option<&autd3_driver::geometry::Transducer> + Send + 'static>,
    >
{
    fn from_msg(msg: &ConfigureDebugOutputIdx) -> Option<Self> {
        let map = msg.value.clone();
        Some(autd3_driver::datagram::ConfigureDebugOutputIdx::new(
            Box::new(move |dev| match map[dev.idx()] {
                -1 => None,
                idx => Some(&dev[idx as usize]),
            }),
        ))
    }
}
