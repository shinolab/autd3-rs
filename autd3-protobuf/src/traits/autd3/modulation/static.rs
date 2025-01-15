use autd3_core::modulation::ModulationProperty;

use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

impl ToMessage for autd3::modulation::Static {
    type Message = Datagram;

    fn to_msg(&self, _: Option<&autd3_core::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram::Datagram::Modulation(Modulation {
                modulation: Some(modulation::Modulation::Static(Static {
                    intensity: Some(self.intensity() as _),
                })),
                loop_behavior: Some(self.loop_behavior().to_msg(None)),
            })),
            timeout: None,
            parallel_threshold: None,
        }
    }
}

impl FromMessage<Static> for autd3::modulation::Static {
    fn from_msg(msg: &Static) -> Result<Self, AUTDProtoBufError> {
        if let Some(intensity) = msg.intensity {
            Ok(Self::with_intensity(intensity as _))
        } else {
            Ok(Self::new())
        }
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
            Some(datagram::Datagram::Modulation(Modulation {
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
