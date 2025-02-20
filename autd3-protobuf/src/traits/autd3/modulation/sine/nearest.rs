use crate::{
    AUTDProtoBufError,
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3::modulation::Sine<autd3::modulation::sampling_mode::Nearest> {
    type Message = Datagram;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(Self::Message {
            datagram: Some(datagram::Datagram::Modulation(Modulation {
                modulation: Some(modulation::Modulation::SineNearest(SineNearest {
                    freq: self.freq.0.hz() as _,
                    option: Some(self.option.to_msg(None)?),
                })),
            })),
        })
    }
}

impl FromMessage<SineNearest>
    for autd3::modulation::Sine<autd3::modulation::sampling_mode::Nearest>
{
    fn from_msg(msg: SineNearest) -> Result<Self, AUTDProtoBufError> {
        Ok(autd3::modulation::Sine {
            freq: msg.freq * autd3_core::defined::Hz,
            option: autd3::modulation::SineOption::from_msg(
                msg.option.ok_or(AUTDProtoBufError::DataParseError)?,
            )?,
        }
        .into_nearest())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3_core::defined::Hz;

    #[test]
    fn test_sine() {
        let m = autd3::modulation::Sine {
            freq: 1. * Hz,
            option: Default::default(),
        }
        .into_nearest();
        let msg = m.to_msg(None).unwrap();
        match msg.datagram {
            Some(datagram::Datagram::Modulation(Modulation {
                modulation: Some(modulation::Modulation::SineNearest(modulation)),
                ..
            })) => {
                let m2 =
                    autd3::modulation::Sine::<autd3::modulation::sampling_mode::Nearest>::from_msg(
                        modulation,
                    )
                    .unwrap();
                assert_eq!(m.freq, m2.freq);
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
