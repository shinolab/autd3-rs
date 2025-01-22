use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

impl ToMessage for autd3::modulation::Square<autd3::modulation::sampling_mode::Nearest> {
    type Message = Datagram;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(Self::Message {
            datagram: Some(datagram::Datagram::Modulation(Modulation {
                modulation: Some(modulation::Modulation::SquareNearest(SquareNearest {
                    freq: self.freq.0.hz() as _,
                    option: Some(self.option.to_msg(None)?),
                })),
            })),
        })
    }
}

impl FromMessage<SquareNearest>
    for autd3::modulation::Square<autd3::modulation::sampling_mode::Nearest>
{
    fn from_msg(msg: &SquareNearest) -> Result<Self, AUTDProtoBufError> {
        Ok(autd3::modulation::Square {
            freq: msg.freq * autd3_core::defined::Hz,
            option: msg
                .option
                .as_ref()
                .map(autd3::modulation::SquareOption::from_msg)
                .transpose()?
                .unwrap_or_default(),
        }
        .into_nearest())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3_core::defined::Hz;

    #[test]
    fn test_square() {
        let m = autd3::modulation::Square {
            freq: 1. * Hz,
            option: Default::default(),
        }
        .into_nearest();
        let msg = m.to_msg(None).unwrap();
        match msg.datagram {
            Some(datagram::Datagram::Modulation(Modulation {
                modulation: Some(modulation::Modulation::SquareNearest(modulation)),
                ..
            })) => {
                let m2 =
                    autd3::modulation::Square::<autd3::modulation::sampling_mode::Nearest>::from_msg(
                        &modulation,
                    )
                    .unwrap();
                assert_eq!(m.freq, m2.freq);
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
