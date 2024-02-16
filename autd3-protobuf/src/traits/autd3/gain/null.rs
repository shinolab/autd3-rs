use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3::gain::Null {
    type Message = DatagramLightweight;

    #[allow(clippy::unnecessary_cast)]
    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram_lightweight::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Null(Null {})),
                segment: Segment::S0 as _,
                update_segment: true,
            })),
        }
    }
}

impl ToMessage for autd3_driver::datagram::DatagramWithSegment<autd3::gain::Null> {
    type Message = DatagramLightweight;

    #[allow(clippy::unnecessary_cast)]
    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram_lightweight::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Null(Null {})),
                segment: self.segment() as _,
                update_segment: self.update_segment(),
            })),
        }
    }
}

impl FromMessage<Null> for autd3::gain::Null {
    #[allow(clippy::unnecessary_cast)]
    fn from_msg(_msg: &Null) -> Option<Self> {
        Some(Self::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bessel() {
        let g = autd3::gain::Null::new();
        let msg = g.to_msg(None);

        match msg.datagram {
            Some(datagram_lightweight::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Null(gain)),
            })) => {
                let _ = autd3::gain::Null::from_msg(&gain).unwrap();
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
