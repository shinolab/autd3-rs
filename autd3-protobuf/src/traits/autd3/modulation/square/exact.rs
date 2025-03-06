use autd3_core::defined::Freq;

use crate::{
    AUTDProtoBufError,
    pb::*,
    traits::{DatagramLightweight, FromMessage},
};

impl DatagramLightweight for autd3::modulation::Square<Freq<u32>> {
    fn into_datagram_lightweight(
        self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Datagram, AUTDProtoBufError> {
        Ok(Datagram {
            datagram: Some(datagram::Datagram::Modulation(Modulation {
                modulation: Some(modulation::Modulation::SquareExact(SquareExact {
                    freq: self.freq.hz() as _,
                    option: Some(self.option.into()),
                })),
            })),
        })
    }
}

impl FromMessage<SquareExact> for autd3::modulation::Square<Freq<u32>> {
    fn from_msg(msg: SquareExact) -> Result<Self, AUTDProtoBufError> {
        Ok(autd3::modulation::Square {
            freq: msg.freq * autd3_core::defined::Hz,
            option: autd3::modulation::SquareOption::from_msg(
                msg.option.ok_or(AUTDProtoBufError::DataParseError)?,
            )?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3_core::defined::Hz;

    #[test]
    fn test_square() {
        let m = autd3::modulation::Square {
            freq: 1 * Hz,
            option: Default::default(),
        };
        let msg = m.clone().into_datagram_lightweight(None).unwrap();
        match msg.datagram {
            Some(datagram::Datagram::Modulation(Modulation {
                modulation: Some(modulation::Modulation::SquareExact(modulation)),
                ..
            })) => {
                let m2 = autd3::modulation::Square::<Freq<u32>>::from_msg(modulation).unwrap();
                assert_eq!(m, m2);
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
