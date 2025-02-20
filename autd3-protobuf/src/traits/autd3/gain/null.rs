use crate::{
    AUTDProtoBufError,
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3::gain::Null {
    type Message = Datagram;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(Self::Message {
            datagram: Some(datagram::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Null(Null {})),
            })),
        })
    }
}

impl FromMessage<Null> for autd3::gain::Null {
    fn from_msg(_msg: Null) -> Result<Self, AUTDProtoBufError> {
        Ok(Self {})
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null() {
        let g = autd3::gain::Null {};
        let msg = g.to_msg(None).unwrap();
        match msg.datagram {
            Some(datagram::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Null(gain)),
                ..
            })) => {
                let _ = autd3::gain::Null::from_msg(gain).unwrap();
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
