use autd3_driver::geometry::{Device, Transducer};

use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl<F: Fn(&Device, &Transducer) -> u16> ToMessage
    for autd3_driver::datagram::ConfigureModDelay<F>
{
    type Message = DatagramLightweight;

    #[allow(clippy::unnecessary_cast)]
    fn to_msg(&self, geometry: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram_lightweight::Datagram::ModDelay(
                ConfigureModDelay {
                    value: geometry
                        .map(|g| {
                            g.iter()
                                .map(|d| configure_mod_delay::Delay {
                                    value: d
                                        .iter()
                                        .map(|tr| (self.f())(d, tr) as u32)
                                        .collect::<Vec<u32>>(),
                                })
                                .collect::<Vec<configure_mod_delay::Delay>>()
                        })
                        .unwrap_or_default(),
                },
            )),
        }
    }
}

impl FromMessage<ConfigureModDelay>
    for autd3_driver::datagram::ConfigureModDelay<
        Box<dyn Fn(&Device, &Transducer) -> u16 + Send + 'static>,
    >
{
    #[allow(clippy::unnecessary_cast)]
    fn from_msg(msg: &ConfigureModDelay) -> Option<Self> {
        let map = msg.value.clone();
        Some(autd3_driver::datagram::ConfigureModDelay::new(Box::new(
            move |dev, tr| map[dev.idx()].value[tr.idx()] as u16,
        )))
    }
}
