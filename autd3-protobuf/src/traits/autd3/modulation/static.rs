use crate::{
    AUTDProtoBufError,
    pb::*,
    traits::{DatagramLightweight, FromMessage},
};

impl DatagramLightweight for autd3::modulation::Static {
    fn into_datagram_lightweight(
        self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Datagram, AUTDProtoBufError> {
        Ok(Datagram {
            datagram: Some(datagram::Datagram::Modulation(Modulation {
                modulation: Some(modulation::Modulation::Static(Static {
                    intensity: Some(self.intensity as _),
                })),
            })),
        })
    }
}

impl FromMessage<Static> for autd3::modulation::Static {
    fn from_msg(msg: Static) -> Result<Self, AUTDProtoBufError> {
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
        let mut rng = rand::rng();

        let m = autd3::modulation::Static {
            intensity: rng.random::<u8>(),
        };
        let msg = m.into_datagram_lightweight(None).unwrap();
        match msg.datagram {
            Some(datagram::Datagram::Modulation(Modulation {
                modulation: Some(modulation::Modulation::Static(modulation)),
                ..
            })) => {
                let m2 = autd3::modulation::Static::from_msg(modulation).unwrap();
                assert_eq!(m.intensity, m2.intensity);
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
