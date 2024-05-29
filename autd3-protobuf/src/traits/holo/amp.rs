use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3_gain_holo::Amplitude {
    type Message = Amplitude;

    #[allow(clippy::unnecessary_cast)]
    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            value: self.pascal() as _,
        }
    }
}

impl FromMessage<Amplitude> for autd3_gain_holo::Amplitude {
    #[allow(clippy::unnecessary_cast)]
    fn from_msg(msg: &Amplitude) -> Option<Self> {
        Some(msg.value as f64 * autd3_gain_holo::Pa)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3_gain_holo::Pa;
    use rand::Rng;

    #[test]
    fn test_amp() {
        let mut rng = rand::thread_rng();
        let v = rng.gen::<f64>() * Pa;
        let msg = v.to_msg(None);
        let v2 = autd3_gain_holo::Amplitude::from_msg(&msg).unwrap();
        assert_approx_eq::assert_approx_eq!(v.pascal(), v2.pascal());
    }
}
