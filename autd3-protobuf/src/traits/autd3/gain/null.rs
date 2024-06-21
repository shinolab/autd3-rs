use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

impl ToMessage for autd3::gain::Null {
    type Message = Datagram;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Null(Null {})),
            })),
            timeout: None,
            parallel_threshold: None,
        }
    }
}

impl FromMessage<Null> for autd3::gain::Null {
    fn from_msg(_msg: &Null) -> Result<Self, AUTDProtoBufError> {
        Ok(Self::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null() {
        let g = autd3::gain::Null::new();
        let msg = g.to_msg(None);

        match msg.datagram {
            Some(datagram::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Null(gain)),
                ..
            })) => {
                let _ = autd3::gain::Null::from_msg(&gain).unwrap();
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
