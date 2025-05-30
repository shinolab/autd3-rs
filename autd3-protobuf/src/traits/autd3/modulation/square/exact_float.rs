use autd3_core::common::Freq;

use crate::{
    AUTDProtoBufError,
    pb::*,
    traits::{DatagramLightweight, FromMessage},
};

impl DatagramLightweight for autd3::modulation::Square<Freq<f32>> {
    fn into_datagram_lightweight(
        self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<RawDatagram, AUTDProtoBufError> {
        Ok(RawDatagram {
            datagram: Some(raw_datagram::Datagram::Modulation(Modulation {
                modulation: Some(modulation::Modulation::SquareExactFloat(SquareExactFloat {
                    freq: self.freq.hz() as _,
                    option: Some(self.option.into()),
                })),
            })),
        })
    }
}

impl FromMessage<SquareExactFloat> for autd3::modulation::Square<Freq<f32>> {
    fn from_msg(msg: SquareExactFloat) -> Result<Self, AUTDProtoBufError> {
        Ok(autd3::modulation::Square {
            freq: msg.freq * autd3_core::common::Hz,
            option: autd3::modulation::SquareOption::from_msg(
                msg.option.ok_or(AUTDProtoBufError::DataParseError)?,
            )?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3_core::common::Hz;

    #[test]
    fn test_square() {
        let m = autd3::modulation::Square {
            freq: 1.0 * Hz,
            option: Default::default(),
        };
        let msg = m.into_datagram_lightweight(None).unwrap();
        match msg.datagram {
            Some(raw_datagram::Datagram::Modulation(Modulation {
                modulation: Some(modulation::Modulation::SquareExactFloat(modulation)),
                ..
            })) => {
                let m2 = autd3::modulation::Square::<Freq<f32>>::from_msg(modulation).unwrap();
                assert_eq!(m, m2);
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
