use autd3_core::defined::Freq;

use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

impl ToMessage for autd3::modulation::Square<Freq<f32>> {
    type Message = Datagram;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(Self::Message {
            datagram: Some(datagram::Datagram::Modulation(Modulation {
                modulation: Some(modulation::Modulation::SquareExactFloat(SquareExactFloat {
                    freq: self.freq.hz() as _,
                    option: Some(self.option.to_msg(None)?),
                })),
            })),
        })
    }
}

impl FromMessage<SquareExactFloat> for autd3::modulation::Square<Freq<f32>> {
    fn from_msg(msg: &SquareExactFloat) -> Result<Self, AUTDProtoBufError> {
        Ok(autd3::modulation::Square {
            freq: msg.freq * autd3_core::defined::Hz,
            option: msg
                .option
                .as_ref()
                .map(autd3::modulation::SquareOption::from_msg)
                .transpose()?
                .unwrap_or_default(),
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
            freq: 1.0 * Hz,
            option: Default::default(),
        };
        let msg = m.to_msg(None).unwrap();
        match msg.datagram {
            Some(datagram::Datagram::Modulation(Modulation {
                modulation: Some(modulation::Modulation::SquareExactFloat(modulation)),
                ..
            })) => {
                let m2 = autd3::modulation::Square::<Freq<f32>>::from_msg(&modulation).unwrap();
                assert_eq!(m.freq, m2.freq);
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
