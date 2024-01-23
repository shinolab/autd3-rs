use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3::modulation::Static {
    type Message = DatagramLightweight;

    #[allow(clippy::unnecessary_cast)]
    fn to_msg(&self) -> Self::Message {
        Self::Message {
            datagram: Some(datagram_lightweight::Datagram::Modulation(Modulation {
                modulation: Some(modulation::Modulation::Static(Static {
                    intensity: Some(self.intensity().to_msg()),
                })),
            })),
        }
    }
}

impl FromMessage<Static> for autd3::modulation::Static {
    #[allow(clippy::unnecessary_cast)]
    fn from_msg(msg: &Static) -> Option<Self> {
        Some(Self::with_intensity(
            autd3_driver::common::EmitIntensity::from_msg(msg.intensity.as_ref()?)?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3_driver::common::EmitIntensity;
    use rand::Rng;

    #[test]
    fn test_sine() {
        let mut rng = rand::thread_rng();

        let m = autd3::modulation::Static::with_intensity(EmitIntensity::new(rng.gen()));
        let msg = m.to_msg();

        match msg.datagram {
            Some(datagram_lightweight::Datagram::Modulation(Modulation {
                modulation: Some(modulation::Modulation::Static(modulation)),
            })) => {
                let m2 = autd3::modulation::Static::from_msg(&modulation).unwrap();
                assert_eq!(m.intensity(), m2.intensity());
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
