use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

impl ToMessage for autd3_gain_holo::Amplitude {
    type Message = Amplitude;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            value: self.pascal() as _,
        }
    }
}

impl FromMessage<Option<Amplitude>> for autd3_gain_holo::Amplitude {
    fn from_msg(msg: &Option<Amplitude>) -> Result<Self, AUTDProtoBufError> {
        msg.map(|msg| msg.value * autd3_gain_holo::Pa)
            .ok_or(AUTDProtoBufError::DataParseError)
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
        let v = rng.gen::<f32>() * Pa;
        let msg = v.to_msg(None);
        let v2 = autd3_gain_holo::Amplitude::from_msg(&Some(msg)).unwrap();
        approx::assert_abs_diff_eq!(v.pascal(), v2.pascal());
    }
}
