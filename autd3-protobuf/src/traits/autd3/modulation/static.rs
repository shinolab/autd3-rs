use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3::modulation::Static {
    type Message = DatagramLightweight;

    #[allow(clippy::unnecessary_cast)]
    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram_lightweight::Datagram::Modulation(Modulation {
                modulation: Some(modulation::Modulation::Static(Static {
                    intensity: self.intensity() as _,
                })),
                segment: Segment::S0 as _,
                transition_mode: Some(TransitionMode::SyncIdx.into()),
                transition_value: Some(0),
            })),
        }
    }
}

impl ToMessage for autd3_driver::datagram::DatagramWithSegment<autd3::modulation::Static> {
    type Message = DatagramLightweight;

    #[allow(clippy::unnecessary_cast)]
    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram_lightweight::Datagram::Modulation(Modulation {
                modulation: Some(modulation::Modulation::Static(Static {
                    intensity: self.intensity() as _,
                })),
                segment: self.segment() as _,
                transition_mode: self.transition_mode().map(|m| m.mode() as _),
                transition_value: self.transition_mode().map(|m| m.value()),
            })),
        }
    }
}

impl FromMessage<Static> for autd3::modulation::Static {
    #[allow(clippy::unnecessary_cast)]
    fn from_msg(msg: &Static) -> Option<Self> {
        Some(Self::with_intensity(msg.intensity as u8))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_sine() {
        let mut rng = rand::thread_rng();

        let m = autd3::modulation::Static::with_intensity(rng.gen::<u8>());
        let msg = m.to_msg(None);

        match msg.datagram {
            Some(datagram_lightweight::Datagram::Modulation(Modulation {
                modulation: Some(modulation::Modulation::Static(modulation)),
                ..
            })) => {
                let m2 = autd3::modulation::Static::from_msg(&modulation).unwrap();
                assert_eq!(m.intensity(), m2.intensity());
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
