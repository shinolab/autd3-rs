use crate::{
    AUTDProtoBufError,
    pb::*,
    traits::{FromMessage, driver::datagram::gain::IntoLightweightGain},
};

impl IntoLightweightGain for autd3::gain::Null {
    fn into_lightweight(self) -> Gain {
        Gain {
            gain: Some(gain::Gain::Null(Null {})),
        }
    }
}

impl FromMessage<Null> for autd3::gain::Null {
    fn from_msg(_msg: Null) -> Result<Self, AUTDProtoBufError> {
        Ok(Self {})
    }
}

#[cfg(test)]
mod tests {
    use crate::DatagramLightweight;

    use super::*;

    #[test]
    fn null() {
        let g = autd3::gain::Null {};
        let msg = g.into_datagram_lightweight(None).unwrap();
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
