use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

impl ToMessage for autd3::modulation::Static {
    type Message = Datagram;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(Self::Message {
            datagram: Some(datagram::Datagram::Modulation(Modulation {
                modulation: Some(modulation::Modulation::Static(Static {
                    intensity: Some(self.intensity as _),
                })),
            })),
        })
    }
}

impl FromMessage<Static> for autd3::modulation::Static {
    fn from_msg(msg: &Static) -> Result<Self, AUTDProtoBufError> {
        Ok(Self {
            intensity: msg
                .intensity
                .map(u8::try_from)
                .transpose()?
                .unwrap_or(autd3::modulation::Static::default().intensity),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_sine() {
        let mut rng = rand::thread_rng();

        let m = autd3::modulation::Static {
            intensity: rng.gen::<u8>(),
        };
        let msg = m.to_msg(None).unwrap();
        match msg.datagram {
            Some(datagram::Datagram::Modulation(Modulation {
                modulation: Some(modulation::Modulation::Static(modulation)),
                ..
            })) => {
                let m2 = autd3::modulation::Static::from_msg(&modulation).unwrap();
                assert_eq!(m.intensity, m2.intensity);
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
